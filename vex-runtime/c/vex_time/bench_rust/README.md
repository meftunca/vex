# Rust Benchmark

Run Rust chrono benchmarks.

## Requirements

- Rust 1.70 or later
- Cargo

## Run

```bash
cargo build --release
cargo run --release
```

## Expected Output

```
══════════════════════════════════════════════════════════
  Rust chrono::DateTime Benchmark
══════════════════════════════════════════════════════════

RFC3339 Parse: 600-800 ns/op
RFC3339 Format: 100-150 ns/op
Custom Format: 150-250 ns/op

══════════════════════════════════════════════════════════
```

