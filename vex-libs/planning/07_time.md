# Vex Stdlib Planning - 07: Time

**Priority:** 7
**Status:** ✅ IMPLEMENTED (native C integration)
**Dependencies:** builtin

## 📦 Packages in This Category

### 7.1 time (native C integration)
**Status:** ✅ FULLY IMPLEMENTED (vex_time C library integration, complete Go-style API)
**Description:** High-performance time handling with timezone support

#### Current Implementation
- ✅ Native vex_time C library integration
- ✅ SIMD-accelerated RFC3339 parsing
- ✅ High-performance monotonic clock
- ✅ Duration parsing and formatting
- ✅ Timezone support
- ✅ Sleep functions with nanosecond precision
- ✅ Complete Go-style time API
- ✅ Comprehensive test suite with 6 detailed test files

#### Implemented API
```vex
// Core types
export struct Duration {
    ns: VexDuration,
}

export struct Instant {
    inner: VexInstant,
}

export struct Time {
    inner: VexTime,
}

export struct Location {
    tz: VexTz,
}

export enum Month {
    January, February, March, April, May, June,
    July, August, September, October, November, December,
}

export enum Weekday {
    Sunday, Monday, Tuesday, Wednesday, Thursday, Friday, Saturday,
}

// Core functions
export fn now(): Time                                    // Current local time ✅
export fn monotonic_now(): u64                           // Monotonic time in nanoseconds ✅
export fn unix(sec: i64, nsec: i32): Time                // Time from Unix timestamp ✅
export fn sleep(d: Duration)                             // Sleep for duration ✅

// Duration operations
export fn parse_duration(s: str): Result<Duration, str>  // Parse duration string ✅
export fn duration_string(d: Duration): str              // Format duration as string ✅

// Arithmetic
export fn add(t: Time, d: Duration): Time                // t + d ✅
export fn sub(t: Time, u: Time): Duration                // t - u (duration) ✅
export fn since(t: Time): Duration                       // Time since t ✅
export fn until(t: Time): Duration                       // Time until t ✅

// RFC3339
export fn format_rfc3339(t: Time): str                   // Format as RFC3339 ✅
export fn parse_rfc3339(s: str): Result<Time, str>       // Parse RFC3339 ✅

// Timezone operations
export fn utc(): Location                                // UTC timezone ✅
export fn local(): Location                              // Local timezone ✅
export fn fixed_zone(name: str, offset: i32): Location   // Fixed offset zone ✅
export fn load_location(name: str): Result<Location, str> // Load IANA timezone ✅

// Go-style formatting (fully implemented)
export fn format(t: Time, layout: str): str              // Format with Go layout ✅
export fn parse(layout: str, value: str): Result<Time, str> // Parse with Go layout ✅

// Additional time methods
export fn unix(t: Time): i64                             // Unix timestamp from Time ✅
export fn unix_nano(t: Time): i64                        // Unix nanoseconds from Time ✅
export fn nanosecond(t: Time): i32                       // Nanoseconds within second ✅

// Duration constants (nanoseconds)
export const NANOSECOND: VexDuration = 1
export const MICROSECOND: VexDuration = 1000
export const MILLISECOND: VexDuration = 1000000
export const SECOND: VexDuration = 1000000000
export const MINUTE: VexDuration = 60000000000
export const HOUR: VexDuration = 3600000000000
```

#### Dependencies
- builtin
- Native vex_time C library (compiled with module)

### 7.2 Additional Time Utilities
**Status:** ❌ Missing (useful extensions)
**Description:** Additional time-related utilities

#### time/rate
```vex
struct Limiter {
    burst: i32,
    rate: f64,
    tokens: f64,
    last: time.Time,
    // mutex
}

fn new_limiter(r: f64, b: i32): *Limiter
fn allow(l: *Limiter): bool
fn allow_n(l: *Limiter, now: time.Time, n: i32): bool
fn reserve(l: *Limiter): *Reservation
fn reserve_n(l: *Limiter, now: time.Time, n: i32): *Reservation
fn set_burst(l: *Limiter, b: i32)
fn set_rate(l: *Limiter, r: f64)
fn tokens(l: *Limiter): f64
```

#### time/tzdata
```vex
// Timezone database access
fn zones(): []str
fn zone_data(name: str): Result<[]u8, Error>
fn load_from_tzdata(data: []u8): Result<*Location, Error>
```

## 🎯 Implementation Priority

1. **time extensions** - Complete core time functionality
2. **time/rate** - Rate limiting utilities
3. **time/tzdata** - Timezone database handling

## ⚠️ Language Feature Issues

- **Channels:** Timer/Ticker use channels - goroutine support needed
- **Global State:** Timezone database may need global state
- **Precision:** Nanosecond precision requirements

## 📋 Missing Critical Dependencies

- **Goroutines:** For timer/ticker implementations
- **Channels:** For time-based communication
- **Atomic Operations:** For concurrent time operations

## 🚀 Next Steps

1. Extend existing time package with full functionality
2. Implement timezone handling
3. Add rate limiting utilities
4. Create timer/ticker with goroutine support