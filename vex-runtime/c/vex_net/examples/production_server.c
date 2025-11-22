// Production-Ready HTTP Server Template
// Demonstrates vex_net best practices for production use
//
// Features:
// - Comprehensive error handling
// - Graceful shutdown
// - Connection lifecycle management  
// - Timeout handling
// - Backpressure management
// - Metrics collection
// - Signal handling
// - Resource cleanup

#include "../include/vex_net.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>
#include <time.h>
#include <signal.h>
#include <errno.h>
#include <sys/time.h>

// Configuration
#define LISTEN_PORT 8080
#define MAX_CLIENTS 10000
#define BUFFER_SIZE 8192
#define CONNECT_TIMEOUT_MS 30000  // 30 seconds
#define IDLE_TIMEOUT_MS 60000     // 60 seconds
#define SHUTDOWN_TIMEOUT_MS 5000  // 5 seconds

// Connection states
typedef enum {
    CONN_ACCEPTING,
    CONN_READING_REQUEST,
    CONN_PROCESSING,
    CONN_WRITING_RESPONSE,
    CONN_CLOSING,
    CONN_CLOSED
} ConnState;

// Connection info
typedef struct {
    int fd;
    ConnState state;
    uint64_t connect_time_ms;
    uint64_t last_activity_ms;
    char read_buf[BUFFER_SIZE];
    char write_buf[BUFFER_SIZE];
    size_t read_pos;
    size_t write_pos;
    size_t bytes_to_write;
    int active;
    char remote_ip[64];
    uint16_t remote_port;
} Connection;

// Server state
typedef struct {
    VexNetLoop loop;
    int listen_fd;
    Connection *connections;
    int max_connections;
    volatile sig_atomic_t running;
    volatile sig_atomic_t shutdown_requested;
    
    // Metrics
    atomic_long total_connections;
    atomic_long active_connections;
    atomic_long bytes_received;
    atomic_long bytes_sent;
    atomic_long requests_handled;
    atomic_long errors;
    atomic_long timeouts;
} Server;

static Server g_server;

// ============================================================================
// Time utilities
// ============================================================================

static uint64_t now_ms(void) {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (uint64_t)tv.tv_sec * 1000 + tv.tv_usec / 1000;
}

// ============================================================================
// Signal handling
// ============================================================================

static void signal_handler(int sig) {
    (void)sig;
    g_server.shutdown_requested = 1;
    g_server.running = 0;
}

static void setup_signals(void) {
    struct sigaction sa;
    memset(&sa, 0, sizeof(sa));
    sa.sa_handler = signal_handler;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = 0;
    
    sigaction(SIGINT, &sa, NULL);
    sigaction(SIGTERM, &sa, NULL);
    
    // Ignore SIGPIPE (handle write errors explicitly)
    signal(SIGPIPE, SIG_IGN);
}

// ============================================================================
// Connection management
// ============================================================================

static Connection* find_connection(int fd) {
    for (int i = 0; i < g_server.max_connections; i++) {
        if (g_server.connections[i].active && g_server.connections[i].fd == fd) {
            return &g_server.connections[i];
        }
    }
    return NULL;
}

static Connection* allocate_connection(void) {
    for (int i = 0; i < g_server.max_connections; i++) {
        if (!g_server.connections[i].active) {
            return &g_server.connections[i];
        }
    }
    return NULL;
}

static void init_connection(Connection *conn, int fd, const char *ip, uint16_t port) {
    memset(conn, 0, sizeof(Connection));
    conn->fd = fd;
    conn->state = CONN_READING_REQUEST;
    conn->active = 1;
    conn->connect_time_ms = now_ms();
    conn->last_activity_ms = conn->connect_time_ms;
    strncpy(conn->remote_ip, ip, sizeof(conn->remote_ip) - 1);
    conn->remote_port = port;
}

static void close_connection(Connection *conn, const char *reason) {
    if (!conn || !conn->active) return;
    
    fprintf(stderr, "[%s:%u] Closing: %s\n", conn->remote_ip, conn->remote_port, reason);
    
    vex_net_unregister(&g_server.loop, conn->fd);
    vex_net_close(conn->fd);
    
    conn->active = 0;
    conn->state = CONN_CLOSED;
    atomic_fetch_sub(&g_server.active_connections, 1);
}

// ============================================================================
// Timeout management
// ============================================================================

static void check_timeouts(void) {
    uint64_t now = now_ms();
    
    for (int i = 0; i < g_server.max_connections; i++) {
        Connection *conn = &g_server.connections[i];
        if (!conn->active) continue;
        
        uint64_t idle_time = now - conn->last_activity_ms;
        
        // Idle timeout
        if (idle_time > IDLE_TIMEOUT_MS) {
            atomic_fetch_add(&g_server.timeouts, 1);
            close_connection(conn, "idle timeout");
            continue;
        }
        
        // Connection timeout (for new connections)
        if (conn->state == CONN_ACCEPTING) {
            uint64_t conn_time = now - conn->connect_time_ms;
            if (conn_time > CONNECT_TIMEOUT_MS) {
                atomic_fetch_add(&g_server.timeouts, 1);
                close_connection(conn, "connect timeout");
            }
        }
    }
}

// ============================================================================
// HTTP request handling (simple example)
// ============================================================================

static void handle_http_request(Connection *conn) {
    // Simple HTTP response
    const char *response = 
        "HTTP/1.1 200 OK\r\n"
        "Content-Type: text/plain\r\n"
        "Content-Length: 13\r\n"
        "Connection: keep-alive\r\n"
        "\r\n"
        "Hello, World!";
    
    size_t len = strlen(response);
    if (len > BUFFER_SIZE) len = BUFFER_SIZE;
    
    memcpy(conn->write_buf, response, len);
    conn->bytes_to_write = len;
    conn->write_pos = 0;
    conn->state = CONN_WRITING_RESPONSE;
    
    atomic_fetch_add(&g_server.requests_handled, 1);
}

// ============================================================================
// Event handlers
// ============================================================================

static void handle_accept(void) {
    // Batch accept
    for (int i = 0; i < 32; i++) {
        char ip[64];
        uint16_t port = 0;
        
        int client_fd = vex_net_accept(g_server.listen_fd, ip, sizeof(ip), &port);
        if (client_fd < 0) break;
        
        // Check connection limit
        Connection *conn = allocate_connection();
        if (!conn) {
            fprintf(stderr, "❌ Connection limit reached, rejecting %s:%u\n", ip, port);
            vex_net_close(client_fd);
            atomic_fetch_add(&g_server.errors, 1);
            continue;
        }
        
        // Configure socket
        vex_net_set_nodelay(client_fd, 1);
        
        // Initialize connection
        init_connection(conn, client_fd, ip, port);
        
        // Register for read events
        if (vex_net_register(&g_server.loop, client_fd, VEX_EVT_READ, (uintptr_t)conn) != 0) {
            fprintf(stderr, "❌ Failed to register client %s:%u\n", ip, port);
            vex_net_close(client_fd);
            conn->active = 0;
            atomic_fetch_add(&g_server.errors, 1);
            continue;
        }
        
        atomic_fetch_add(&g_server.total_connections, 1);
        atomic_fetch_add(&g_server.active_connections, 1);
        
        fprintf(stderr, "✅ Accepted connection from %s:%u (fd=%d)\n", ip, port, client_fd);
    }
}

static void handle_read(Connection *conn) {
    conn->last_activity_ms = now_ms();
    
    ssize_t n = vex_net_read(conn->fd, 
                             conn->read_buf + conn->read_pos,
                             BUFFER_SIZE - conn->read_pos - 1);
    
    if (n < 0) {
        if (errno == EAGAIN || errno == EWOULDBLOCK) {
            return; // Would block, normal
        }
        fprintf(stderr, "[%s:%u] Read error: %s\n", 
                conn->remote_ip, conn->remote_port, strerror(errno));
        atomic_fetch_add(&g_server.errors, 1);
        close_connection(conn, "read error");
        return;
    }
    
    if (n == 0) {
        // Clean connection close
        close_connection(conn, "client closed");
        return;
    }
    
    atomic_fetch_add(&g_server.bytes_received, n);
    conn->read_pos += n;
    conn->read_buf[conn->read_pos] = '\0';
    
    // Simple HTTP request detection (look for \r\n\r\n)
    if (strstr(conn->read_buf, "\r\n\r\n")) {
        // Got complete request
        handle_http_request(conn);
        
        // Switch to write events
        vex_net_modify(&g_server.loop, conn->fd, VEX_EVT_WRITE, (uintptr_t)conn);
    } else if (conn->read_pos >= BUFFER_SIZE - 1) {
        // Request too large
        fprintf(stderr, "[%s:%u] Request too large\n", conn->remote_ip, conn->remote_port);
        atomic_fetch_add(&g_server.errors, 1);
        close_connection(conn, "request too large");
    }
}

static void handle_write(Connection *conn) {
    conn->last_activity_ms = now_ms();
    
    size_t remaining = conn->bytes_to_write - conn->write_pos;
    if (remaining == 0) {
        // Done writing, go back to reading
        conn->read_pos = 0;
        conn->state = CONN_READING_REQUEST;
        vex_net_modify(&g_server.loop, conn->fd, VEX_EVT_READ, (uintptr_t)conn);
        return;
    }
    
    ssize_t n = vex_net_write(conn->fd, 
                              conn->write_buf + conn->write_pos,
                              remaining);
    
    if (n < 0) {
        if (errno == EAGAIN || errno == EWOULDBLOCK) {
            return; // Would block, normal
        }
        if (errno == EPIPE || errno == ECONNRESET) {
            close_connection(conn, "client disconnected");
            return;
        }
        fprintf(stderr, "[%s:%u] Write error: %s\n", 
                conn->remote_ip, conn->remote_port, strerror(errno));
        atomic_fetch_add(&g_server.errors, 1);
        close_connection(conn, "write error");
        return;
    }
    
    atomic_fetch_add(&g_server.bytes_sent, n);
    conn->write_pos += n;
}

// ============================================================================
// Main server loop
// ============================================================================

static void print_stats(void) {
    fprintf(stderr, "\n═══════════════════════════════════════════════════════════\n");
    fprintf(stderr, "  SERVER STATISTICS\n");
    fprintf(stderr, "═══════════════════════════════════════════════════════════\n");
    fprintf(stderr, "  Total connections:  %ld\n", atomic_load(&g_server.total_connections));
    fprintf(stderr, "  Active connections: %ld\n", atomic_load(&g_server.active_connections));
    fprintf(stderr, "  Requests handled:   %ld\n", atomic_load(&g_server.requests_handled));
    fprintf(stderr, "  Bytes received:     %.2f MB\n", 
            atomic_load(&g_server.bytes_received) / 1024.0 / 1024.0);
    fprintf(stderr, "  Bytes sent:         %.2f MB\n", 
            atomic_load(&g_server.bytes_sent) / 1024.0 / 1024.0);
    fprintf(stderr, "  Errors:             %ld\n", atomic_load(&g_server.errors));
    fprintf(stderr, "  Timeouts:           %ld\n", atomic_load(&g_server.timeouts));
    fprintf(stderr, "═══════════════════════════════════════════════════════════\n\n");
}

int main(void) {
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Production HTTP Server (vex_net)\n");
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Port: %d\n", LISTEN_PORT);
    printf("  Max connections: %d\n", MAX_CLIENTS);
    printf("  Idle timeout: %d seconds\n", IDLE_TIMEOUT_MS / 1000);
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    // Initialize server
    memset(&g_server, 0, sizeof(g_server));
    g_server.max_connections = MAX_CLIENTS;
    g_server.running = 1;
    
    // Allocate connection pool
    g_server.connections = calloc(MAX_CLIENTS, sizeof(Connection));
    if (!g_server.connections) {
        fprintf(stderr, "❌ Failed to allocate connection pool\n");
        return 1;
    }
    
    // Setup signal handlers
    setup_signals();
    
    // Create event loop
    if (vex_net_loop_create(&g_server.loop) != 0) {
        fprintf(stderr, "❌ Failed to create event loop\n");
        return 1;
    }
    
    // Create and configure listening socket
    g_server.listen_fd = vex_net_socket_tcp(0);
    if (g_server.listen_fd < 0) {
        fprintf(stderr, "❌ Failed to create socket\n");
        return 1;
    }
    
    if (vex_net_bind(g_server.listen_fd, "0.0.0.0", LISTEN_PORT, 1, 1, 0) != 0) {
        fprintf(stderr, "❌ Failed to bind to port %d\n", LISTEN_PORT);
        return 1;
    }
    
    if (vex_net_listen(g_server.listen_fd, 1024) != 0) {
        fprintf(stderr, "❌ Failed to listen\n");
        return 1;
    }
    
    if (vex_net_register(&g_server.loop, g_server.listen_fd, VEX_EVT_READ, 
                        (uintptr_t)g_server.listen_fd) != 0) {
        fprintf(stderr, "❌ Failed to register listen socket\n");
        return 1;
    }
    
    printf("✅ Server started successfully!\n");
    printf("   Listening on 0.0.0.0:%d\n", LISTEN_PORT);
    printf("   Press Ctrl+C to shutdown gracefully\n\n");
    
    VexEvent events[256];
    uint64_t last_timeout_check = now_ms();
    uint64_t last_stats = now_ms();
    
    // Main event loop
    while (g_server.running) {
        int nevents = vex_net_tick(&g_server.loop, events, 256, 1000);
        
        if (nevents < 0) {
            fprintf(stderr, "❌ Event loop error\n");
            break;
        }
        
        // Process events
        for (int i = 0; i < nevents; i++) {
            VexEvent *ev = &events[i];
            
            // Listen socket
            if (ev->userdata == (uintptr_t)g_server.listen_fd) {
                handle_accept();
                continue;
            }
            
            // Client connection
            Connection *conn = (Connection *)ev->userdata;
            if (!conn || !conn->active) continue;
            
            // Error or hangup
            if (ev->events & (VEX_EVT_HUP | VEX_EVT_ERR)) {
                close_connection(conn, "error/hangup");
                continue;
            }
            
            // Read event
            if (ev->events & VEX_EVT_READ) {
                handle_read(conn);
            }
            
            // Write event
            if (ev->events & VEX_EVT_WRITE) {
                handle_write(conn);
            }
        }
        
        // Periodic timeout check (every second)
        uint64_t now = now_ms();
        if (now - last_timeout_check >= 1000) {
            check_timeouts();
            last_timeout_check = now;
        }
        
        // Print stats every 10 seconds
        if (now - last_stats >= 10000) {
            print_stats();
            last_stats = now;
        }
    }
    
    // Graceful shutdown
    printf("\n🛑 Shutting down gracefully...\n");
    
    // Stop accepting new connections
    vex_net_unregister(&g_server.loop, g_server.listen_fd);
    vex_net_close(g_server.listen_fd);
    
    // Close all active connections
    uint64_t shutdown_start = now_ms();
    int remaining = 0;
    do {
        remaining = 0;
        for (int i = 0; i < MAX_CLIENTS; i++) {
            if (g_server.connections[i].active) {
                close_connection(&g_server.connections[i], "server shutdown");
                remaining++;
            }
        }
        
        if (remaining > 0 && (now_ms() - shutdown_start) > SHUTDOWN_TIMEOUT_MS) {
            fprintf(stderr, "⚠️  Forced shutdown, %d connections remaining\n", remaining);
            break;
        }
    } while (remaining > 0);
    
    // Cleanup
    vex_net_loop_close(&g_server.loop);
    free(g_server.connections);
    
    // Final stats
    print_stats();
    
    printf("✅ Shutdown complete\n");
    return 0;
}
