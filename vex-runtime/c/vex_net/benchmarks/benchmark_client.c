// Connection-Pooled Load Generator for vex_net Benchmarking
// Reuses connections to eliminate TCP handshake overhead
#include "../include/vex_net.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>
#include <time.h>
#include <unistd.h>

#define SERVER_HOST "127.0.0.1"
#define SERVER_PORT 19999
#define NUM_CONNECTIONS 100        // Connection pool size
#define MESSAGES_PER_CONN 100      // Messages per connection
#define TOTAL_MESSAGES (NUM_CONNECTIONS * MESSAGES_PER_CONN)
#define MESSAGE_SIZE 64
#define BATCH_EVENTS 256

// Statistics
static atomic_int messages_sent = 0;
static atomic_int messages_received = 0;
static atomic_int connections_established = 0;
static atomic_int connections_active = 0;

typedef enum {
    STATE_CONNECTING,
    STATE_SENDING,
    STATE_RECEIVING,
    STATE_DONE
} ConnState;

typedef struct {
    int fd;
    ConnState state;
    int messages_sent_local;
    int messages_received_local;
    char send_buf[MESSAGE_SIZE];
    char recv_buf[MESSAGE_SIZE];
    size_t recv_pos;
    int active;
} Connection;

static Connection connections[NUM_CONNECTIONS];

static void init_connection(Connection *conn, int id) {
    memset(conn, 0, sizeof(Connection));
    conn->fd = -1;
    conn->state = STATE_CONNECTING;
    snprintf(conn->send_buf, MESSAGE_SIZE, "MSG_%04d_", id);
}

static int start_connect(VexNetLoop *loop, Connection *conn) {
    conn->fd = vex_net_socket_tcp(0);
    if (conn->fd < 0) {
        fprintf(stderr, "❌ Failed to create socket\n");
        return -1;
    }

    vex_net_set_nodelay(conn->fd, 1);

    int rc = vex_net_connect(conn->fd, SERVER_HOST, SERVER_PORT);
    if (rc == 0) {
        // Connected immediately
        conn->state = STATE_SENDING;
        conn->active = 1;
        atomic_fetch_add(&connections_established, 1);
        atomic_fetch_add(&connections_active, 1);
        vex_net_register(loop, conn->fd, VEX_EVT_WRITE, (uintptr_t)conn);
        return 0;
    } else if (rc == -2) {
        // In progress (EINPROGRESS)
        conn->active = 1;
        atomic_fetch_add(&connections_active, 1);
        vex_net_register(loop, conn->fd, VEX_EVT_WRITE, (uintptr_t)conn);
        return 0;
    }

    vex_net_close(conn->fd);
    conn->fd = -1;
    return -1;
}

static void close_connection(VexNetLoop *loop, Connection *conn) {
    if (conn->active && conn->fd >= 0) {
        vex_net_unregister(loop, conn->fd);
        vex_net_close(conn->fd);
        atomic_fetch_sub(&connections_active, 1);
    }
    conn->fd = -1;
    conn->active = 0;
}

int main(void) {
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  vex_net Load Generator (Connection Pooling)\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Target: %s:%d\n", SERVER_HOST, SERVER_PORT);
    printf("  Connections: %d\n", NUM_CONNECTIONS);
    printf("  Messages per connection: %d\n", MESSAGES_PER_CONN);
    printf("  Total messages: %d\n", TOTAL_MESSAGES);
    printf("  Message size: %d bytes\n", MESSAGE_SIZE);
    printf("═══════════════════════════════════════════════════════════\n\n");

    // Initialize connections
    for (int i = 0; i < NUM_CONNECTIONS; i++) {
        init_connection(&connections[i], i);
    }

    // Create event loop
    VexNetLoop loop;
    if (vex_net_loop_create(&loop) != 0) {
        fprintf(stderr, "❌ Failed to create event loop\n");
        return 1;
    }

    // Start all connections
    printf("📡 Establishing connections...\n");
    for (int i = 0; i < NUM_CONNECTIONS; i++) {
        if (start_connect(&loop, &connections[i]) != 0) {
            fprintf(stderr, "❌ Failed to start connection %d\n", i);
        }
    }

    struct timespec start_time, end_time;
    clock_gettime(CLOCK_MONOTONIC, &start_time);

    VexEvent events[BATCH_EVENTS];
    int all_done = 0;

    printf("🚀 Starting benchmark...\n\n");

    // Event loop
    while (!all_done) {
        int nevents = vex_net_tick(&loop, events, BATCH_EVENTS, 1000);
        
        if (nevents < 0) {
            fprintf(stderr, "❌ Event loop error\n");
            break;
        }

        for (int i = 0; i < nevents; i++) {
            VexEvent *ev = &events[i];
            Connection *conn = (Connection *)ev->userdata;

            if (!conn || !conn->active) continue;

            // Error or hangup
            if (ev->events & (VEX_EVT_HUP | VEX_EVT_ERR)) {
                fprintf(stderr, "❌ Connection error/hangup\n");
                close_connection(&loop, conn);
                continue;
            }

            switch (conn->state) {
            case STATE_CONNECTING:
                // Check if connected
                if (ev->events & VEX_EVT_WRITE) {
                    conn->state = STATE_SENDING;
                    atomic_fetch_add(&connections_established, 1);
                    // Fall through to sending
                }
                break;

            case STATE_SENDING:
                if (ev->events & VEX_EVT_WRITE) {
                    // Batch up to 32 messages with writev
                    #define WRITEV_BATCH 32
                    struct vex_iovec iov[WRITEV_BATCH];
                    int batch_count = 0;
                    
                    while (conn->messages_sent_local < MESSAGES_PER_CONN && batch_count < WRITEV_BATCH) {
                        iov[batch_count].base = conn->send_buf;
                        iov[batch_count].len = MESSAGE_SIZE;
                        batch_count++;
                        conn->messages_sent_local++;
                    }
                    
                    if (batch_count > 0) {
                        ssize_t n = vex_net_writev(conn->fd, iov, batch_count);
                        if (n > 0) {
                            int msgs = (int)(n / MESSAGE_SIZE);
                            atomic_fetch_add(&messages_sent, msgs);
                        } else if (n == -1) {
                            // Would block, adjust state
                            conn->messages_sent_local -= batch_count;
                            break;
                        } else {
                            // Error
                            close_connection(&loop, conn);
                            goto next_event;
                        }
                    }

                    if (conn->messages_sent_local >= MESSAGES_PER_CONN) {
                        // All sent, start receiving
                        conn->state = STATE_RECEIVING;
                        vex_net_modify(&loop, conn->fd, VEX_EVT_READ, (uintptr_t)conn);
                    }
                }
                break;

            case STATE_RECEIVING:
                if (ev->events & VEX_EVT_READ) {
                    while (conn->messages_received_local < MESSAGES_PER_CONN) {
                        ssize_t n = vex_net_read(conn->fd, 
                                                conn->recv_buf + conn->recv_pos,
                                                MESSAGE_SIZE - conn->recv_pos);
                        if (n <= 0) {
                            if (n == 0) {
                                // Connection closed prematurely
                                close_connection(&loop, conn);
                                goto next_event;
                            }
                            break; // Would block
                        }

                        conn->recv_pos += n;
                        if (conn->recv_pos >= MESSAGE_SIZE) {
                            conn->messages_received_local++;
                            atomic_fetch_add(&messages_received, 1);
                            conn->recv_pos = 0;
                        }
                    }

                    if (conn->messages_received_local >= MESSAGES_PER_CONN) {
                        // Done!
                        conn->state = STATE_DONE;
                        close_connection(&loop, conn);
                    }
                }
                break;

            case STATE_DONE:
                break;
            }
next_event:;
        }

        // Check if all done
        all_done = 1;
        for (int i = 0; i < NUM_CONNECTIONS; i++) {
            if (connections[i].state != STATE_DONE) {
                all_done = 0;
                break;
            }
        }

        // Progress
        int sent = atomic_load(&messages_sent);
        int received = atomic_load(&messages_received);
        fprintf(stderr, "\rProgress: %d/%d sent, %d/%d received", 
                sent, TOTAL_MESSAGES, received, TOTAL_MESSAGES);
        fflush(stderr);
    }

    clock_gettime(CLOCK_MONOTONIC, &end_time);

    double elapsed = (end_time.tv_sec - start_time.tv_sec) + 
                    (end_time.tv_nsec - start_time.tv_nsec) / 1e9;

    int sent_total = atomic_load(&messages_sent);
    int received_total = atomic_load(&messages_received);
    int established = atomic_load(&connections_established);

    printf("\n\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  BENCHMARK RESULTS\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Connections established: %d/%d\n", established, NUM_CONNECTIONS);
    printf("  Messages sent: %d/%d\n", sent_total, TOTAL_MESSAGES);
    printf("  Messages received: %d/%d\n", received_total, TOTAL_MESSAGES);
    printf("  Time elapsed: %.3f seconds\n", elapsed);
    printf("  Throughput: %.0f msg/s\n", received_total / elapsed);
    printf("  Latency (avg): %.2f ms/msg\n", (elapsed / received_total) * 1000);
    printf("  Bandwidth: %.2f MB/s\n", 
           (received_total * MESSAGE_SIZE) / elapsed / 1024.0 / 1024.0);
    printf("═══════════════════════════════════════════════════════════\n");

    double throughput = received_total / elapsed;
    if (throughput >= 100000) {
        printf("✅ EXCELLENT: >100K msg/s achieved!\n");
    } else if (throughput >= 50000) {
        printf("✅ GREAT: >50K msg/s\n");
    } else if (throughput >= 10000) {
        printf("✅ GOOD: >10K msg/s\n");
    } else if (throughput >= 1000) {
        printf("⚠️  MODERATE: >1K msg/s\n");
    } else {
        printf("❌ LOW: <1K msg/s - check server/network\n");
    }

    vex_net_loop_close(&loop);
    return 0;
}
