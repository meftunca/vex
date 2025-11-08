# Vex Time — Cross‑Platform C‑ABI

Go `time` paketine benzer bir yüzey: `Time` (wall+monotonic), `Duration`, `Timer`, `Ticker`, RFC3339 ve Duration parse/format.

## Derleme
### Make
```bash
make
./examples/time_demo
```

### CMake
```bash
cmake -B build -S .
cmake --build build --config Release
./build/time_demo
```

## API
- `vt_now()` — UTC wall + monotonic ns
- `vt_parse_duration`, `vt_format_duration`
- `vt_format_rfc3339_utc`, `vt_parse_rfc3339`
- Scheduler: `vt_sched_*` + `vt_timer_*` + `vt_ticker_*` (tek arka plan thread, min-heap)

## Notlar
- `vt_sleep_ns` POSIX’te `nanosleep`, Windows’ta **high-resolution waitable timer** kullanır.
- RFC3339 parse: `YYYY-MM-DDTHH:MM:SS[.frac](Z|±HH:MM)`.
- Duration parse: `h m s ms us µs ns` birimlerini ve ondalıkları destekler.
