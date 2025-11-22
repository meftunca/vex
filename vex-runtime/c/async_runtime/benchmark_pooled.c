// Connection-Pooled Stress Test - Network I/O Optimization
// Reuses connections for multiple messages to eliminate setup/teardown overhead
#include <errno.h>
#include "../include/runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/internal.h"
#include <stdatomic.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <fcntl.h>
#include <time.h>

#define TEST_PORT 19999
#define TOTAL_MESSAGES 10000
#define MESSAGES_PER_CONN 100    // Reuse each connection for 100 messages!
#define NUM_CONNECTIONS (TOTAL_MESSAGES / MESSAGES_PER_CONN)
#define MSG_SIZE 64
#define CONCURRENT_LIMIT 100     // Fewer concurrent connections needed

static atomic_int messages_sent = 0;
static atomic_int messages_received = 0;
static atomic_int clients_done = 0;
static atomic_int active_clients = 0;
static int server_fd = -1;
static _Atomic(bool) server_ready = false;

static int set_nonblocking(int fd)
{
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags == -1)
        return -1;
    return fcntl(fd, F_SETFL, flags | O_NONBLOCK);
}

typedef struct
{
    int client_fd;
    char buffer[MSG_SIZE];
    int bytes_transferred;
    int messages_handled;
    int state; // 0=read, 1=write
} ServerConn;

static CoroStatus server_coro(WorkerContext *ctx, void *data)
{
    ServerConn *conn = (ServerConn *)data;

    while (conn->messages_handled < MESSAGES_PER_CONN)
    {
        switch (conn->state)
        {
        case 0: // Read message
        {
            ssize_t n = read(conn->client_fd, conn->buffer + conn->bytes_transferred,
                           MSG_SIZE - conn->bytes_transferred);
            if (n > 0)
            {
                conn->bytes_transferred += n;
                if (conn->bytes_transferred >= MSG_SIZE)
                {
                    conn->state = 1; // Switch to write
                    conn->bytes_transferred = 0;
                    return CORO_STATUS_RUNNING;
                }
                return CORO_STATUS_RUNNING;
            }
            else if (n == 0 || (n < 0 && errno != EAGAIN && errno != EWOULDBLOCK))
            {
                goto cleanup;
            }
            worker_await_io(ctx, conn->client_fd, EVENT_TYPE_READABLE);
            return CORO_STATUS_YIELDED;
        }

        case 1: // Write echo response
        {
            ssize_t n = write(conn->client_fd, conn->buffer + conn->bytes_transferred,
                            MSG_SIZE - conn->bytes_transferred);
            if (n > 0)
            {
                conn->bytes_transferred += n;
                if (conn->bytes_transferred >= MSG_SIZE)
                {
                    atomic_fetch_add(&messages_received, 1);
                    conn->messages_handled++;
                    conn->state = 0; // Back to read for next message
                    conn->bytes_transferred = 0;
                    
                    // Continue immediately to next message
                    return CORO_STATUS_RUNNING;
                }
                return CORO_STATUS_RUNNING;
            }
            else if (n < 0 && errno != EAGAIN && errno != EWOULDBLOCK)
            {
                goto cleanup;
            }
            worker_await_io(ctx, conn->client_fd, EVENT_TYPE_WRITABLE);
            return CORO_STATUS_YIELDED;
        }
        }
    }

cleanup:
    close(conn->client_fd);
    xfree(conn);
    return CORO_STATUS_DONE;
}

static CoroStatus accept_coro(WorkerContext *ctx, void *data)
{
    (void)data;
    atomic_store(&server_ready, true);

    // Accept in batches
    while (1)
    {
        int accepted = 0;
        while (accepted < 64)
        {
            struct sockaddr_in addr;
            socklen_t len = sizeof(addr);
            int client_fd = accept(server_fd, (struct sockaddr *)&addr, &len);

            if (client_fd >= 0)
            {
                set_nonblocking(client_fd);
                ServerConn *conn = (ServerConn *)xmalloc(sizeof(ServerConn));
                memset(conn, 0, sizeof(ServerConn));
                conn->client_fd = client_fd;
                runtime_spawn_global(ctx->owner->rt, server_coro, conn);
                accepted++;
            }
            else if (errno == EAGAIN || errno == EWOULDBLOCK)
            {
                break;
            }
        }

        if (atomic_load(&messages_received) >= TOTAL_MESSAGES)
        {
            return CORO_STATUS_DONE;
        }

        worker_await_io(ctx, server_fd, EVENT_TYPE_READABLE);
        return CORO_STATUS_YIELDED;
    }
}

typedef struct
{
    int id;
    int sock;
    char buffer[MSG_SIZE];
    int bytes_transferred;
    int messages_sent_count;
    int messages_received_count;
    int state; // 0=connect, 1=check_connect, 2=write, 3=read
} ClientTask;

static CoroStatus client_coro(WorkerContext *ctx, void *data)
{
    ClientTask *ct = (ClientTask *)data;

    switch (ct->state)
    {
    case 0: // Create socket and start connect
    {
        atomic_fetch_add(&active_clients, 1);

        ct->sock = socket(AF_INET, SOCK_STREAM, 0);
        if (ct->sock < 0)
        {
            goto cleanup;
        }
        set_nonblocking(ct->sock);

        struct sockaddr_in addr;
        memset(&addr, 0, sizeof(addr));
        addr.sin_family = AF_INET;
        addr.sin_port = htons(TEST_PORT);
        inet_pton(AF_INET, "127.0.0.1", &addr.sin_addr);

        int rc = connect(ct->sock, (struct sockaddr *)&addr, sizeof(addr));
        if (rc == 0)
        {
            ct->state = 2; // Connected immediately, go to write
            return CORO_STATUS_RUNNING;
        }
        else if (errno == EINPROGRESS)
        {
            ct->state = 1; // Wait for connection
            worker_await_io(ctx, ct->sock, EVENT_TYPE_WRITABLE);
            return CORO_STATUS_YIELDED;
        }
        goto cleanup;
    }

    case 1: // Check connection result
    {
        int error = 0;
        socklen_t len = sizeof(error);
        getsockopt(ct->sock, SOL_SOCKET, SO_ERROR, &error, &len);
        if (error != 0)
        {
            goto cleanup;
        }
        ct->state = 2; // Connected, start sending
        return CORO_STATUS_RUNNING;
    }

    case 2: // Write message (pipelined - send MESSAGES_PER_CONN messages)
    {
        while (ct->messages_sent_count < MESSAGES_PER_CONN)
        {
            // Prepare message
            snprintf(ct->buffer, MSG_SIZE, "MSG_%d_%d", ct->id, ct->messages_sent_count);

            ssize_t n = write(ct->sock, ct->buffer + ct->bytes_transferred,
                            MSG_SIZE - ct->bytes_transferred);
            if (n > 0)
            {
                ct->bytes_transferred += n;
                if (ct->bytes_transferred >= MSG_SIZE)
                {
                    atomic_fetch_add(&messages_sent, 1);
                    ct->messages_sent_count++;
                    ct->bytes_transferred = 0;
                    // Continue sending next message
                    continue;
                }
                return CORO_STATUS_RUNNING;
            }
            else if (n < 0 && errno != EAGAIN && errno != EWOULDBLOCK)
            {
                goto cleanup;
            }

            worker_await_io(ctx, ct->sock, EVENT_TYPE_WRITABLE);
            return CORO_STATUS_YIELDED;
        }

        // All messages sent, switch to read
        ct->state = 3;
        ct->bytes_transferred = 0;
        return CORO_STATUS_RUNNING;
    }

    case 3: // Read responses (read MESSAGES_PER_CONN responses)
    {
        while (ct->messages_received_count < MESSAGES_PER_CONN)
        {
            ssize_t n = read(ct->sock, ct->buffer + ct->bytes_transferred,
                           MSG_SIZE - ct->bytes_transferred);
            if (n > 0)
            {
                ct->bytes_transferred += n;
                if (ct->bytes_transferred >= MSG_SIZE)
                {
                    ct->messages_received_count++;
                    ct->bytes_transferred = 0;
                    // Continue reading next response
                    continue;
                }
                return CORO_STATUS_RUNNING;
            }
            else if (n == 0 || (n < 0 && errno != EAGAIN && errno != EWOULDBLOCK))
            {
                goto cleanup;
            }

            worker_await_io(ctx, ct->sock, EVENT_TYPE_READABLE);
            return CORO_STATUS_YIELDED;
        }

        // All messages sent and received
        atomic_fetch_add(&clients_done, 1);
        goto cleanup;
    }
    }

cleanup:
    atomic_fetch_sub(&active_clients, 1);
    if (ct->sock >= 0)
        close(ct->sock);
    xfree(ct);
    return CORO_STATUS_DONE;
}

static CoroStatus spawner_coro(WorkerContext *ctx, void *data)
{
    static int spawned = 0;
    (void)data;

    // Wait for server to be ready
    if (!atomic_load(&server_ready))
    {
        return CORO_STATUS_RUNNING;
    }

    // Spawn all clients at once (simplified approach)
    if (spawned == 0)
    {
        for (int i = 0; i < NUM_CONNECTIONS; i++)
        {
            ClientTask *ct = (ClientTask *)xmalloc(sizeof(ClientTask));
            memset(ct, 0, sizeof(ClientTask));
            ct->id = i;
            ct->sock = -1;
            runtime_spawn_global(ctx->owner->rt, client_coro, ct);
        }
        spawned = 1;
    }

    return CORO_STATUS_DONE;
}

static CoroStatus monitor_coro(WorkerContext *ctx, void *data)
{
    (void)ctx;
    (void)data;

    static time_t last_report = 0;
    time_t now = time(NULL);

    if (now > last_report)
    {
        int sent = atomic_load(&messages_sent);
        int received = atomic_load(&messages_received);
        int active = atomic_load(&active_clients);

        fprintf(stderr, "\r[%ld sec] Progress: %d/%d sent, %d/%d received, %d active",
                now - last_report, sent, TOTAL_MESSAGES, received, TOTAL_MESSAGES, active);
        fflush(stderr);
        last_report = now;
    }

    int done = atomic_load(&clients_done);
    if (done >= NUM_CONNECTIONS)
    {
        fprintf(stderr, "\n✅ All clients done!\n");
        return CORO_STATUS_DONE;
    }

    return CORO_STATUS_RUNNING;
}

int main(void)
{
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Connection-Pooled Network Benchmark\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Total messages: %d\n", TOTAL_MESSAGES);
    printf("  Messages per connection: %d\n", MESSAGES_PER_CONN);
    printf("  Number of connections: %d\n", NUM_CONNECTIONS);
    printf("  Concurrent limit: %d\n", CONCURRENT_LIMIT);
    printf("  Optimization: Connection reuse (vs 1 msg per conn)\n");
    printf("═══════════════════════════════════════════════════════════\n\n");

    // Create server socket
    server_fd = socket(AF_INET, SOCK_STREAM, 0);
    int reuse = 1;
    setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, &reuse, sizeof(reuse));
    set_nonblocking(server_fd);

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(TEST_PORT);

    bind(server_fd, (struct sockaddr *)&addr, sizeof(addr));
    listen(server_fd, 1024);

    printf("Server listening on port %d\n\n", TEST_PORT);

    Runtime *rt = runtime_create(4);
    runtime_enable_auto_shutdown(rt, false);

    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

    // Spawn server, clients, and monitor
    runtime_spawn_global(rt, accept_coro, NULL);
    runtime_spawn_global(rt, spawner_coro, NULL);
    runtime_spawn_global(rt, monitor_coro, NULL);

    runtime_run(rt);

    clock_gettime(CLOCK_MONOTONIC, &end);

    double elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;

    int sent = atomic_load(&messages_sent);
    int received = atomic_load(&messages_received);

    printf("\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  RESULTS\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Messages sent: %d/%d\n", sent, TOTAL_MESSAGES);
    printf("  Messages received: %d/%d\n", received, TOTAL_MESSAGES);
    printf("  Time: %.3f seconds\n", elapsed);
    printf("  Throughput: %.0f msg/sec\n", received / elapsed);
    printf("  Latency: %.2f ms/msg\n", (elapsed / received) * 1000);
    printf("═══════════════════════════════════════════════════════════\n");

    double throughput = received / elapsed;
    if (throughput >= 100000)
    {
        printf("✅ EXCELLENT: >100K msg/s achieved!\n");
    }
    else if (throughput >= 10000)
    {
        printf("✅ GOOD: >10K msg/s (10x improvement over baseline)\n");
    }
    else if (throughput >= 1000)
    {
        printf("⚠️  MODERATE: >1K msg/s\n");
    }
    else
    {
        printf("❌ LOW: <1K msg/s\n");
    }

    runtime_destroy(rt);
    close(server_fd);

    return 0;
}
