# Time Package

High-precision timing and sleep functions.

## Functions

```vex
fn now(): i64           // Current Unix timestamp (seconds)
fn high_res(): i64      // High-resolution timer (nanoseconds)
fn sleep_ms(ms: i64)    // Sleep for milliseconds
```

## Usage

```vex
import { time } from "std";

fn main(): i32 {
    let start = time.high_res();

    // Do work...
    compute();

    let elapsed = time.high_res() - start;
    print("Elapsed: ");
    print(elapsed);
    println("ns");

    time.sleep_ms(1000);  // Sleep 1 second

    return 0;
}
```

## C Runtime

Uses `vex-clibs/vex_time.c`:

- `vex_time_now_sec()`
- `vex_time_high_res()`
- `vex_sleep_ms()`

## Precision

- **macOS/Linux:** `clock_gettime()` with `CLOCK_MONOTONIC` (~1ns)
- **Windows:** `QueryPerformanceCounter()` (~100ns)

## Status

⚠️ **Placeholder implementations** - C runtime integration pending.
