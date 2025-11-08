# Go Benchmark

Run Go time.Parse() and time.Format() benchmarks.

## Requirements

- Go 1.21 or later

## Run

```bash
go run time_bench.go
```

## Expected Output

```
══════════════════════════════════════════════════════════
  Go time.Parse() Benchmark
══════════════════════════════════════════════════════════

RFC3339 Parse: 1500-2000 ns/op
RFC3339 Format: 150-200 ns/op
Layout Parse (RFC1123Z): 2000-2500 ns/op
Layout Format (RFC1123Z): 200-300 ns/op

══════════════════════════════════════════════════════════
```

