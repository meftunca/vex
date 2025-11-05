// vex_time.c - Time and date operations
#include "vex.h"
#include <time.h>
#include <sys/time.h>

// ============================================================================
// TIME OPERATIONS
// ============================================================================

int64_t vex_time_now() {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    
    // Return Unix timestamp in milliseconds
    return (int64_t)tv.tv_sec * 1000 + (int64_t)tv.tv_usec / 1000;
}

int64_t vex_time_now_micros() {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    
    // Return Unix timestamp in microseconds
    return (int64_t)tv.tv_sec * 1000000 + (int64_t)tv.tv_usec;
}

int64_t vex_time_now_nanos() {
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    
    // Return Unix timestamp in nanoseconds
    return (int64_t)ts.tv_sec * 1000000000 + (int64_t)ts.tv_nsec;
}

int64_t vex_time_monotonic() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    
    // Return monotonic time in nanoseconds (for measuring durations)
    return (int64_t)ts.tv_sec * 1000000000 + (int64_t)ts.tv_nsec;
}

void vex_time_sleep(int64_t millis) {
    if (millis <= 0) return;
    
    struct timespec ts;
    ts.tv_sec = millis / 1000;
    ts.tv_nsec = (millis % 1000) * 1000000;
    
    nanosleep(&ts, NULL);
}

void vex_time_sleep_micros(int64_t micros) {
    if (micros <= 0) return;
    
    struct timespec ts;
    ts.tv_sec = micros / 1000000;
    ts.tv_nsec = (micros % 1000000) * 1000;
    
    nanosleep(&ts, NULL);
}

// ============================================================================
// DATE/TIME FORMATTING
// ============================================================================

VexDateTime* vex_time_to_datetime(int64_t timestamp_millis) {
    time_t seconds = timestamp_millis / 1000;
    struct tm* tm_info = gmtime(&seconds);  // UTC
    
    if (!tm_info) return NULL;
    
    VexDateTime* dt = (VexDateTime*)vex_malloc(sizeof(VexDateTime));
    dt->year = tm_info->tm_year + 1900;
    dt->month = tm_info->tm_mon + 1;
    dt->day = tm_info->tm_mday;
    dt->hour = tm_info->tm_hour;
    dt->minute = tm_info->tm_min;
    dt->second = tm_info->tm_sec;
    dt->millisecond = (int)(timestamp_millis % 1000);
    dt->weekday = tm_info->tm_wday;  // 0=Sunday
    dt->yearday = tm_info->tm_yday + 1;  // 1-366
    
    return dt;
}

VexDateTime* vex_time_to_local_datetime(int64_t timestamp_millis) {
    time_t seconds = timestamp_millis / 1000;
    struct tm* tm_info = localtime(&seconds);  // Local time
    
    if (!tm_info) return NULL;
    
    VexDateTime* dt = (VexDateTime*)vex_malloc(sizeof(VexDateTime));
    dt->year = tm_info->tm_year + 1900;
    dt->month = tm_info->tm_mon + 1;
    dt->day = tm_info->tm_mday;
    dt->hour = tm_info->tm_hour;
    dt->minute = tm_info->tm_min;
    dt->second = tm_info->tm_sec;
    dt->millisecond = (int)(timestamp_millis % 1000);
    dt->weekday = tm_info->tm_wday;
    dt->yearday = tm_info->tm_yday + 1;
    
    return dt;
}

int64_t vex_datetime_to_timestamp(const VexDateTime* dt) {
    if (!dt) {
        vex_panic("vex_datetime_to_timestamp: NULL datetime");
    }
    
    struct tm tm_info;
    tm_info.tm_year = dt->year - 1900;
    tm_info.tm_mon = dt->month - 1;
    tm_info.tm_mday = dt->day;
    tm_info.tm_hour = dt->hour;
    tm_info.tm_min = dt->minute;
    tm_info.tm_sec = dt->second;
    tm_info.tm_isdst = -1;  // Auto determine DST
    
    time_t seconds = mktime(&tm_info);
    if (seconds == -1) return -1;
    
    return (int64_t)seconds * 1000 + dt->millisecond;
}

char* vex_time_format(const VexDateTime* dt, const char* format) {
    if (!dt || !format) {
        vex_panic("vex_time_format: NULL parameter");
    }
    
    struct tm tm_info;
    tm_info.tm_year = dt->year - 1900;
    tm_info.tm_mon = dt->month - 1;
    tm_info.tm_mday = dt->day;
    tm_info.tm_hour = dt->hour;
    tm_info.tm_min = dt->minute;
    tm_info.tm_sec = dt->second;
    tm_info.tm_wday = dt->weekday;
    tm_info.tm_yday = dt->yearday - 1;
    tm_info.tm_isdst = -1;
    
    char buffer[256];
    size_t len = strftime(buffer, sizeof(buffer), format, &tm_info);
    
    if (len == 0) return NULL;
    
    return vex_strdup(buffer);
}

void vex_datetime_free(VexDateTime* dt) {
    if (dt) {
        vex_free(dt);
    }
}

// ============================================================================
// HIGH-RESOLUTION TIMER (for benchmarking)
// ============================================================================

VexTimer* vex_timer_start() {
    VexTimer* timer = (VexTimer*)vex_malloc(sizeof(VexTimer));
    timer->start_ns = vex_time_monotonic();
    timer->is_running = true;
    
    return timer;
}

int64_t vex_timer_elapsed_nanos(const VexTimer* timer) {
    if (!timer) {
        vex_panic("vex_timer_elapsed_nanos: NULL timer");
    }
    
    int64_t now = vex_time_monotonic();
    return now - timer->start_ns;
}

int64_t vex_timer_elapsed_micros(const VexTimer* timer) {
    return vex_timer_elapsed_nanos(timer) / 1000;
}

int64_t vex_timer_elapsed_millis(const VexTimer* timer) {
    return vex_timer_elapsed_nanos(timer) / 1000000;
}

double vex_timer_elapsed_seconds(const VexTimer* timer) {
    return (double)vex_timer_elapsed_nanos(timer) / 1000000000.0;
}

void vex_timer_reset(VexTimer* timer) {
    if (!timer) {
        vex_panic("vex_timer_reset: NULL timer");
    }
    
    timer->start_ns = vex_time_monotonic();
}

void vex_timer_free(VexTimer* timer) {
    if (timer) {
        vex_free(timer);
    }
}
