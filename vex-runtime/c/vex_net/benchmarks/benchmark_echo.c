// High-Performance Echo Server Benchmark
// Demonstrates vex_net capabilities: io_uring, connection pooling, vectored I/O
#include "../include/vex_net.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>
#include <time.h>
#include <signal.h>

#define LISTEN_PORT 19999
#define MAX_CLIENTS 1024
#define BUFFER_SIZE 4096
#define MESSAGE_SIZE 64
#define BATCH_ACCEPT 64
#define BATCH_EVENTS 256

// Statistics
static atomic_int total_connections = 0;
static atomic_long total_bytes_received = 0;
static atomic_long total_bytes_sent = 0;
static atomic_long total_messages = 0;
static volatile sig_atomic_t running = 1;

typedef struct {
    int fd;
    char read_buf[BUFFER_SIZE];
    size_t read_pos;
    int active;
} Client;

static Client clients[MAX_CLIENTS];

static void signal_handler(int sig) {
    (void)sig;
    running = 0;
}

static void print_stats(time_t start_time) {
    time_t now = time(NULL);
    time_t elapsed = now - start_time;
    if (elapsed == 0) elapsed = 1;

    long total_msgs = atomic_load(&total_messages);
    long total_rx = atomic_load(&total_bytes_received);
    long total_tx = atomic_load(&total_bytes_sent);
    int conns = atomic_load(&total_connections);

    fprintf(stderr, "\r[%ld sec] Msgs: %ld (%ld/s) | RX: %.2f MB | TX: %.2f MB | Conns: %d",
            elapsed,
            total_msgs,
            total_msgs / elapsed,
            total_rx / 1024.0 / 1024.0,
            total_tx / 1024.0 / 1024.0,
            conns);
    fflush(stderr);
}

static Client* find_free_client(void) {
    for (int i = 0; i < MAX_CLIENTS; i++) {
        if (!clients[i].active) {
            return &clients[i];
        }
    }
    return NULL;
}

static void close_client(VexNetLoop *loop, Client *c) {
    if (c->active) {
        vex_net_unregister(loop, c->fd);
        vex_net_close(c->fd);
        c->active = 0;
        c->read_pos = 0;
    }
}

int main(void) {
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  vex_net High-Performance Echo Server\n");
    printf("═══════════════════════════════════════════════════════════\n");

    // Check capabilities
    int caps = vex_net_capabilities();
    printf("  Backend capabilities:\n");
    if (caps & VEX_CAP_IOURING)   printf("    ✅ io_uring (ultra-fast!)\n");
    if (caps & VEX_CAP_KQUEUE)    printf("    ✅ kqueue\n");
    if (caps & VEX_CAP_EPOLLEXCL) printf("    ✅ epoll (EPOLLEXCLUSIVE)\n");
    if (caps & VEX_CAP_UDP_GSO)   printf("    ✅ UDP GSO\n");
    if (caps & VEX_CAP_MSG_ZC)    printf("    ✅ MSG_ZEROCOPY\n");
    
    printf("\n  Listening on: 0.0.0.0:%d\n", LISTEN_PORT);
    printf("  Max clients: %d\n", MAX_CLIENTS);
    printf("  Batch accept: %d\n", BATCH_ACCEPT);
    printf("═══════════════════════════════════════════════════════════\n\n");

    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);

    // Initialize clients
    memset(clients, 0, sizeof(clients));

    // Create event loop
    VexNetLoop loop;
    if (vex_net_loop_create(&loop) != 0) {
        fprintf(stderr, "❌ Failed to create event loop\n");
        return 1;
    }

    // Create listening socket
    int listen_fd = vex_net_socket_tcp(0);
    if (listen_fd < 0) {
        fprintf(stderr, "❌ Failed to create socket\n");
        return 1;
    }

    vex_net_set_nodelay(listen_fd, 1);
    
    if (vex_net_bind(listen_fd, "0.0.0.0", LISTEN_PORT, 1, 1, 0) != 0) {
        fprintf(stderr, "❌ Failed to bind to port %d\n", LISTEN_PORT);
        return 1;
    }

    if (vex_net_listen(listen_fd, 1024) != 0) {
        fprintf(stderr, "❌ Failed to listen\n");
        return 1;
    }

    // Register listen socket
    if (vex_net_register(&loop, listen_fd, VEX_EVT_READ, (uintptr_t)listen_fd) != 0) {
        fprintf(stderr, "❌ Failed to register listen socket\n");
        return 1;
    }

    printf("✅ Server started successfully!\n\n");

    VexEvent events[BATCH_EVENTS];
    time_t start_time = time(NULL);
    time_t last_stats = start_time;

    // Event loop
    while (running) {
        int nevents = vex_net_tick(&loop, events, BATCH_EVENTS, 1000);
        
        if (nevents < 0) {
            fprintf(stderr, "❌ Event loop error\n");
            break;
        }

        // Print stats every second
        time_t now = time(NULL);
        if (now > last_stats) {
            print_stats(start_time);
            last_stats = now;
        }

        for (int i = 0; i < nevents; i++) {
            VexEvent *ev = &events[i];
            int fd = (int)ev->userdata;

            // New connection(s)
            if (fd == listen_fd) {
                // Batch accept
                for (int j = 0; j < BATCH_ACCEPT; j++) {
                    char ip[64];
                    uint16_t port = 0;
                    int client_fd = vex_net_accept(listen_fd, ip, sizeof(ip), &port);
                    
                    if (client_fd < 0) break; // No more pending

                    Client *c = find_free_client();
                    if (!c) {
                        // No space, close immediately
                        vex_net_close(client_fd);
                        continue;
                    }

                    // Setup client
                    vex_net_set_nodelay(client_fd, 1);
                    c->fd = client_fd;
                    c->active = 1;
                    c->read_pos = 0;

                    vex_net_register(&loop, client_fd, VEX_EVT_READ, (uintptr_t)client_fd);
                    atomic_fetch_add(&total_connections, 1);
                }
            }
            // Client event
            else {
                // Find client
                Client *c = NULL;
                for (int j = 0; j < MAX_CLIENTS; j++) {
                    if (clients[j].active && clients[j].fd == fd) {
                        c = &clients[j];
                        break;
                    }
                }

                if (!c) continue; // Client not found (should not happen)

                // Error or hangup
                if (ev->events & (VEX_EVT_HUP | VEX_EVT_ERR)) {
                    close_client(&loop, c);
                    continue;
                }

                // Read data
                if (ev->events & VEX_EVT_READ) {
                    ssize_t n = vex_net_read(c->fd, 
                                            c->read_buf + c->read_pos,
                                            BUFFER_SIZE - c->read_pos);
                    
                    if (n <= 0) {
                        close_client(&loop, c);
                        continue;
                    }

                    atomic_fetch_add(&total_bytes_received, n);
                    c->read_pos += n;

                    // Echo back using writev for batching (split into 64-byte chunks)
                    if (c->read_pos >= MESSAGE_SIZE) {
                        int num_messages = c->read_pos / MESSAGE_SIZE;
                        
                        // Batch with writev (up to 64 messages)
                        #define MAX_BATCH 64
                        struct vex_iovec iov[MAX_BATCH];
                        int batch_size = num_messages < MAX_BATCH ? num_messages : MAX_BATCH;
                        
                        for (int j = 0; j < batch_size; j++) {
                            iov[j].base = c->read_buf + (j * MESSAGE_SIZE);
                            iov[j].len = MESSAGE_SIZE;
                        }
                        
                        ssize_t written = vex_net_writev(c->fd, iov, batch_size);
                        
                        if (written > 0) {
                            atomic_fetch_add(&total_bytes_sent, written);
                            int msgs_sent = (int)(written / MESSAGE_SIZE);
                            atomic_fetch_add(&total_messages, msgs_sent);
                            
                            // Shift remaining data
                            int bytes_written = msgs_sent * MESSAGE_SIZE;
                            if (bytes_written < (int)c->read_pos) {
                                memmove(c->read_buf, c->read_buf + bytes_written, 
                                       c->read_pos - bytes_written);
                                c->read_pos -= bytes_written;
                            } else {
                                c->read_pos = 0;
                            }
                        } else if (written < 0) {
                            close_client(&loop, c);
                        }
                    }
                }
            }
        }
    }

    // Cleanup
    printf("\n\n🛑 Shutting down...\n");
    
    for (int i = 0; i < MAX_CLIENTS; i++) {
        if (clients[i].active) {
            close_client(&loop, &clients[i]);
        }
    }

    vex_net_unregister(&loop, listen_fd);
    vex_net_close(listen_fd);
    vex_net_loop_close(&loop);

    // Final stats
    time_t total_time = time(NULL) - start_time;
    if (total_time == 0) total_time = 1;

    printf("\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  FINAL STATISTICS\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Runtime: %ld seconds\n", total_time);
    printf("  Total connections: %d\n", atomic_load(&total_connections));
    printf("  Total messages: %ld\n", atomic_load(&total_messages));
    printf("  Throughput: %ld msg/s\n", atomic_load(&total_messages) / total_time);
    printf("  Received: %.2f MB\n", atomic_load(&total_bytes_received) / 1024.0 / 1024.0);
    printf("  Sent: %.2f MB\n", atomic_load(&total_bytes_sent) / 1024.0 / 1024.0);
    printf("═══════════════════════════════════════════════════════════\n");

    return 0;
}
