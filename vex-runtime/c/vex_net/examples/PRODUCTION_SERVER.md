# Production Server Example

Production-ready HTTP server demonstrating vex_net best practices.

## Features

### Core Production Patterns
- ✅ Comprehensive error handling (EAGAIN, EPIPE, ECONNRESET, etc.)
- ✅ Graceful shutdown (SIGINT, SIGTERM)
- ✅ Connection lifecycle management
- ✅ Timeout handling (idle, connect)
- ✅ Backpressure management
- ✅ Metrics collection
- ✅ Signal handling (SIGPIPE ignored)
- ✅ Resource cleanup
- ✅ Connection limits

### Implementation Details
- **Connection pooling**: Pre-allocated array, O(1) lookup
- **Batch accept**: Accept up to 32 connections per event
- **Timeout checking**: Periodic scan every second
- **Metrics**: Atomic counters for thread-safe stats
- **Graceful shutdown**: 5-second timeout for active connections

## Building

```bash
cd examples
cc -O3 -Wall -Wextra -std=c11 -o production_server production_server.c \
   ../src/*.c ../src/backends/kqueue.c -I../include
```

## Running

```bash
./production_server
```

Server listens on `0.0.0.0:8080` by default.

## Testing

### Basic test:
```bash
curl http://localhost:8080/
```

### Load test (using Apache Bench):
```bash
ab -n 100000 -c 100 http://localhost:8080/
```

### Expected performance:
- macOS (kqueue): 50K-100K req/s
- Linux (epoll): 100K-500K req/s  
- Linux (io_uring): 500K-1M+ req/s

## Configuration

Edit these constants in `production_server.c`:

```c
#define LISTEN_PORT 8080
#define MAX_CLIENTS 10000
#define BUFFER_SIZE 8192
#define CONNECT_TIMEOUT_MS 30000  // 30 seconds
#define IDLE_TIMEOUT_MS 60000     // 60 seconds
#define SHUTDOWN_TIMEOUT_MS 5000  // 5 seconds
```

## Key Patterns Demonstrated

### 1. Error Handling
```c
ssize_t n = vex_net_read(fd, buf, len);
if (n < 0) {
    if (errno == EAGAIN || errno == EWOULDBLOCK) {
        return; // Would block, normal for non-blocking I/O
    }
    if (errno == EPIPE || errno == ECONNRESET) {
        close_connection(conn, "client disconnected");
        return;
    }
    // Other errors
    atomic_fetch_add(&errors, 1);
    close_connection(conn, "read error");
}
```

### 2. Timeout Management
```c
void check_timeouts(void) {
    uint64_t now = now_ms();
    for (each connection) {
        uint64_t idle = now - conn->last_activity_ms;
        if (idle > IDLE_TIMEOUT_MS) {
            close_connection(conn, "idle timeout");
        }
    }
}
```

### 3. Graceful Shutdown
```c
void signal_handler(int sig) {
    g_server.shutdown_requested = 1;
    g_server.running = 0;
}

// In main loop exit:
vex_net_unregister(&loop, listen_fd);
vex_net_close(listen_fd);

for (each connection) {
    close_connection(conn, "server shutdown");
}
```

### 4. Connection Lifecycle
```c
// States:
CONN_ACCEPTING         // Just accepted
CONN_READING_REQUEST   // Reading HTTP request
CONN_PROCESSING        // Processing (could be async)
CONN_WRITING_RESPONSE  // Writing HTTP response
CONN_CLOSING           // Graceful close
CONN_CLOSED            // Fully closed
```

### 5. Metrics Collection
```c
typedef struct {
    atomic_long total_connections;
    atomic_long active_connections;
    atomic_long bytes_received;
    atomic_long bytes_sent;
    atomic_long requests_handled;
    atomic_long errors;
    atomic_long timeouts;
} ServerMetrics;
```

## Production Considerations

### Must Add for Real Production:
1. **TLS/SSL support** (using OpenSSL/BoringSSL)
2. **Request parsing** (proper HTTP parser)
3. **Response buffering** (for large responses)
4. **Compression** (gzip, brotli)
5. **Logging** (structured logging to file/syslog)
6. **Configuration file** (not hardcoded constants)
7. **Hot reload** (SIGHUP for config reload)
8. **Health checks** (designated endpoint)
9. **Rate limiting** (per-IP, per-endpoint)
10. **Access control** (authentication, authorization)

### Performance Tuning:
```c
// Increase connection limit
#define MAX_CLIENTS 100000

// Tune timeouts based on workload
#define IDLE_TIMEOUT_MS 300000  // 5 minutes for long-polling

// Increase buffers for high throughput
#define BUFFER_SIZE 65536
vex_net_set_recvbuf(fd, 256*1024);
vex_net_set_sendbuf(fd, 256*1024);

// Enable TCP optimizations
vex_net_set_nodelay(fd, 1);
vex_net_set_keepalive(fd, 1, 30, 10, 3);
```

### Multi-threading Pattern:
```c
// One event loop per CPU core
int num_cores = sysconf(_SC_NPROCESSORS_ONLN);
pthread_t workers[num_cores];

for (int i = 0; i < num_cores; i++) {
    VexNetLoop *loop = malloc(sizeof(VexNetLoop));
    vex_net_loop_create(loop);
    pthread_create(&workers[i], NULL, worker_thread, loop);
}

// Use SO_REUSEPORT for load balancing
vex_net_bind(listen_fd, "0.0.0.0", port, 1, 1, 0);
```

## Architecture

```
                     ┌─────────────┐
                     │Listen Socket│
                     └──────┬──────┘
                            │
                     ┌──────▼──────┐
                     │ Event Loop  │
                     │  (vex_net)  │
                     └──────┬──────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
         ┌────▼───┐    ┌────▼───┐   ┌────▼───┐
         │Conn 1  │    │Conn 2  │   │Conn N  │
         │READ/   │    │READ/   │   │READ/   │
         │WRITE   │    │WRITE   │   │WRITE   │
         └────────┘    └────────┘   └────────┘
              │             │             │
         ┌────▼─────────────▼─────────────▼────┐
         │     Connection Pool (Array)          │
         │   - State tracking                   │
         │   - Timeout management                │
         │   - Resource cleanup                  │
         └──────────────────────────────────────┘
```

## Monitoring

Server prints stats every 10 seconds:
```
═══════════════════════════════════════════════════════════
  SERVER STATISTICS
═══════════════════════════════════════════════════════════
  Total connections:  15234
  Active connections: 87
  Requests handled:   89421
  Bytes received:     12.34 MB
  Bytes sent:         45.67 MB
  Errors:             3
  Timeouts:           12
═══════════════════════════════════════════════════════════
```

For production, export to:
- Prometheus (metrics endpoint)
- StatsD (push metrics)
- CloudWatch (if on AWS)
- Datadog (if using Datadog)

## License

Same as vex_net (part of vex-runtime project).
