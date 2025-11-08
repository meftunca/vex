/*
 * Vex Runtime Integration Demo
 * 
 * Demonstrates vex_time + async_runtime + vex_net working together:
 * - vex_time: Time formatting, duration parsing, scheduled callbacks
 * - async_runtime: Coroutine-based async tasks
 * - vex_net: Network event loop (unified with async_runtime)
 * 
 * Scenario: HTTP health checker with scheduled reports
 */

#include "async_runtime/include/runtime.h"
#include "vex_time/include/vex_time.h"
#include "vex_net/include/vex_net.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>

// === Shared State ===
typedef struct {
    atomic_int checks_done;
    atomic_int checks_success;
    atomic_int checks_failed;
    VexTimeSched* time_sched;
    Runtime* async_rt;
} SharedStats;

static SharedStats g_stats;

// === vex_time Callback: Periodic Report ===
static void on_report_tick(void* user, VexTime when) {
    (void)user;
    
    char timestamp[64];
    vt_format_rfc3339_utc(when.wall, timestamp, sizeof(timestamp));
    
    int done = atomic_load(&g_stats.checks_done);
    int success = atomic_load(&g_stats.checks_success);
    int failed = atomic_load(&g_stats.checks_failed);
    
    printf("\n[vex_time Report @ %s]\n", timestamp);
    printf("  Health checks: %d total, %d success, %d failed\n", 
           done, success, failed);
    printf("  Success rate: %.1f%%\n\n", 
           done > 0 ? (success * 100.0 / done) : 0.0);
}

// === async_runtime Task: HTTP Health Check ===
typedef struct {
    const char* host;
    const char* port;
    int check_id;
    int state;  // State machine: 0=connect, 1=send, 2=recv, 3=done
    int fd;
    char recv_buf[512];
} HealthCheckState;

static CoroStatus health_check_coro(WorkerContext* ctx, void* data) {
    HealthCheckState* hc = (HealthCheckState*)data;
    
    switch (hc->state) {
    case 0: { // Connect
        hc->fd = vex_net_socket_tcp(0);
        if (hc->fd < 0) {
            printf("[check %d] Socket creation failed\n", hc->check_id);
            atomic_fetch_add(&g_stats.checks_done, 1);
            atomic_fetch_add(&g_stats.checks_failed, 1);
            free(hc);
            return CORO_STATUS_DONE;
        }
        
        if (vex_net_connect(hc->fd, hc->host, atoi(hc->port)) < 0) {
            // Non-blocking, will complete later
        }
        
        hc->state = 1;
        worker_await_io(ctx, hc->fd, EVENT_TYPE_WRITABLE);
        return CORO_STATUS_YIELDED;
    }
    
    case 1: { // Send request
        const char* request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        ssize_t sent = vex_net_write(hc->fd, request, strlen(request));
        
        if (sent < 0) {
            printf("[check %d] Send failed\n", hc->check_id);
            vex_net_close(hc->fd);
            atomic_fetch_add(&g_stats.checks_done, 1);
            atomic_fetch_add(&g_stats.checks_failed, 1);
            free(hc);
            return CORO_STATUS_DONE;
        }
        
        hc->state = 2;
        worker_await_io(ctx, hc->fd, EVENT_TYPE_READABLE);
        return CORO_STATUS_YIELDED;
    }
    
    case 2: { // Receive response
        ssize_t received = vex_net_read(hc->fd, hc->recv_buf, sizeof(hc->recv_buf) - 1);
        
        if (received > 0) {
            hc->recv_buf[received] = '\0';
            
            // Check for HTTP 200
            int is_success = (strstr(hc->recv_buf, "HTTP/1.") != NULL) &&
                           (strstr(hc->recv_buf, "200") != NULL);
            
            if (is_success) {
                printf("[check %d] ✓ SUCCESS - %s:%s is healthy\n", 
                       hc->check_id, hc->host, hc->port);
                atomic_fetch_add(&g_stats.checks_success, 1);
            } else {
                printf("[check %d] ✗ FAILED - unexpected response\n", hc->check_id);
                atomic_fetch_add(&g_stats.checks_failed, 1);
            }
        } else {
            printf("[check %d] ✗ FAILED - no response\n", hc->check_id);
            atomic_fetch_add(&g_stats.checks_failed, 1);
        }
        
        atomic_fetch_add(&g_stats.checks_done, 1);
        vex_net_close(hc->fd);
        free(hc);
        return CORO_STATUS_DONE;
    }
    }
    
    return CORO_STATUS_DONE;
}

// === Supervisor: Spawn health checks ===
typedef struct {
    int checks_spawned;
    int total_checks;
} SupervisorState;

static CoroStatus supervisor_coro(WorkerContext* ctx, void* data) {
    SupervisorState* ss = (SupervisorState*)data;
    
    if (ss->checks_spawned >= ss->total_checks) {
        // All checks spawned, wait for completion
        int done = atomic_load(&g_stats.checks_done);
        
        if (done >= ss->total_checks) {
            printf("\n[Supervisor] All checks complete, shutting down\n");
            runtime_shutdown(g_stats.async_rt);
            free(ss);
            return CORO_STATUS_DONE;
        }
        
        // Wait a bit more
        worker_await_after(ctx, 100);
        return CORO_STATUS_YIELDED;
    }
    
    // Spawn a health check
    HealthCheckState* hc = (HealthCheckState*)malloc(sizeof(HealthCheckState));
    hc->host = "127.0.0.1";
    hc->port = "8080";
    hc->check_id = ss->checks_spawned;
    hc->state = 0;
    hc->fd = -1;
    
    runtime_spawn_global(g_stats.async_rt, health_check_coro, hc);
    ss->checks_spawned++;
    
    // Spawn next check after delay
    worker_await_after(ctx, 200);
    return CORO_STATUS_YIELDED;
}

int main(void) {
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  Vex Runtime Integration Demo\n");
    printf("  vex_time + async_runtime + vex_net\n");
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    // Initialize
    memset(&g_stats, 0, sizeof(g_stats));
    
    // === 1. vex_time: Demonstration ===
    printf("1. vex_time Demo:\n");
    
    VexTime now;
    vt_now(&now);
    
    char now_str[64];
    vt_format_rfc3339_utc(now.wall, now_str, sizeof(now_str));
    printf("   Current time: %s\n", now_str);
    
    VexDuration duration;
    vt_parse_duration("5s", &duration);
    
    char dur_str[64];
    vt_format_duration(duration, dur_str, sizeof(dur_str));
    printf("   Parsed duration '5s': %s (%lld ns)\n", dur_str, (long long)duration);
    
    // Load timezone
    VexTz* tz = vt_tz_load("America/New_York");
    if (tz) {
        char ny_time[128];
        vt_format_go(now.wall, tz, "Monday, Jan 02 2006 15:04:05 MST", ny_time, sizeof(ny_time));
        printf("   New York time: %s\n", ny_time);
        vt_tz_release(tz);
    }
    
    printf("\n");
    
    // === 2. vex_time Scheduler: Periodic Reports ===
    printf("2. Starting vex_time periodic reporter (every 2s)...\n\n");
    
    g_stats.time_sched = vt_sched_create();
    VexTicker* reporter = vt_ticker_create(g_stats.time_sched, on_report_tick, NULL);
    vt_ticker_start(reporter, 2000LL * 1000 * 1000);  // 2 seconds in nanoseconds
    
    // === 3. async_runtime + vex_net: Health Checks ===
    printf("3. Starting async_runtime health checker...\n\n");
    
    g_stats.async_rt = runtime_create(2);
    runtime_enable_auto_shutdown(g_stats.async_rt, false);
    runtime_set_tracing(g_stats.async_rt, false);
    
    // Spawn supervisor
    SupervisorState* ss = (SupervisorState*)malloc(sizeof(SupervisorState));
    ss->checks_spawned = 0;
    ss->total_checks = 5;  // Total health checks to perform
    runtime_spawn_global(g_stats.async_rt, supervisor_coro, ss);
    
    printf("Note: This demo expects an HTTP server on localhost:8080\n");
    printf("      Start one with: python3 -m http.server 8080\n\n");
    
    // Run async runtime (blocking)
    printf("Running async runtime...\n\n");
    runtime_run(g_stats.async_rt);
    
    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  Demo Complete!\n");
    printf("═══════════════════════════════════════════════════════════\n\n");
    
    // Cleanup
    vt_ticker_stop(reporter);
    vt_ticker_destroy(reporter);
    vt_sched_destroy(g_stats.time_sched);
    runtime_destroy(g_stats.async_rt);
    
    // Final stats
    printf("Final Statistics:\n");
    printf("  Total checks: %d\n", atomic_load(&g_stats.checks_done));
    printf("  Success: %d\n", atomic_load(&g_stats.checks_success));
    printf("  Failed: %d\n", atomic_load(&g_stats.checks_failed));
    
    return 0;
}

