# timevex (C runtime for VEX)

A lightweight C date/time runtime inspired by Go's `time` package.
It covers most day-to-day features: `Duration`, `Time` (with monotonic stamp),
RFC3339 parsing/formatting (incl. nano), time arithmetic, truncation/rounding,
calendar fields, ISO week, sleep, timers/tickers, and basic IANA TZ handling.

> NOTE: Full Go-style custom layouts (using the reference time 2006-01-02 15:04:05) are **not** implemented here. Use RFC3339 helpers or add your own layout layer in VEX.

## Files
- `timevex.h` — public API
- `timevex.c` — implementation
- `example.c` — quick demo

## Build
```sh
cc -std=c11 -O2 -pthread -c timevex.c
ar rcs libtimevex.a timevex.o
# example
cc -std=c11 -O2 -pthread example.c -L. -ltimevex -o example
./example
```

## Go feature coverage
- ✅ `Now`, `Unix`, `UnixNano`/`Seconds` equivalents
- ✅ `Duration` nanoseconds + parser/formatter (Go syntax)
- ✅ `Add`, `AddDate`, `Sub`, `Since`, `Until`
- ✅ `Before`, `After`, `Equal`
- ✅ `Truncate`, `Round`
- ✅ Calendar: `Year`, `Month`, `Day`, `Hour`, `Minute`, `Second`, `Nanosecond`, `Weekday`, `YearDay`, `ISOWeek`
- ✅ RFC3339 / RFC3339Nano parse/format
- ✅ Sleep
- ✅ Timers (`NewTimer/Reset/Stop`) and Tickers (`NewTicker/Reset/Stop`)
- ✅ Locations: `UTC`, `Local`, `LoadLocation("Area/City")`, `FixedZone`
- ⚠️ Setting IANA zones uses a **process-global TZ swap** protected by a mutex for conversions. This is portable but not reentrant across *other* libraries also playing with `TZ`.
- ❌ Full Go layouts (`Format`/`Parse` with `2006-01-02 15:04:05`) — omitted to keep C code compact. You can implement a small layout layer on top if needed.
- ❌ Leap seconds beyond what libc exposes are not modeled explicitly (same as most systems).

## Threading model
Timers & tickers run on pthread worker threads and invoke your callback from those threads. Stopping/freeing is safe. Avoid heavy work in callbacks, or hand off to your runtime scheduler.

## Windows
Replace `clock_gettime`/`nanosleep` with `QueryPerformanceCounter`/`Sleep` or link a shim (e.g. `-lrt` on older glibc).

## Integrating with VEX
- Compile `libtimevex.a` and link it into your runtime.
- Expose thin VEX FFI stubs that call functions in `timevex.h`.
- The API was chosen to be FFI-friendly (plain integers + POD structs).

## License
Public Domain / CC0.
