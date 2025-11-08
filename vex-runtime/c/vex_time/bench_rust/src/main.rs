use chrono::{DateTime, Utc};
use std::time::Instant;

fn main() {
    // RFC3339 Parse Benchmark
    let test_str = "2024-11-07T12:34:56.123456789Z";
    let iterations = 1_000_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _: DateTime<Utc> = test_str.parse().unwrap();
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;

    println!("══════════════════════════════════════════════════════════");
    println!("  Rust chrono::DateTime Benchmark");
    println!("══════════════════════════════════════════════════════════\n");
    println!("RFC3339 Parse: {:.1} ns/op ({:.1}M ops/s)\n", ns_per_op, 1000.0 / ns_per_op);

    // RFC3339 Format Benchmark
    let t: DateTime<Utc> = test_str.parse().unwrap();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = t.to_rfc3339();
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;

    println!("RFC3339 Format: {:.1} ns/op ({:.1}M ops/s)\n", ns_per_op, 1000.0 / ns_per_op);

    // Custom format benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = t.format("%Y-%m-%d %H:%M:%S").to_string();
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;

    println!("Custom Format: {:.1} ns/op ({:.1}M ops/s)", ns_per_op, 1000.0 / ns_per_op);

    println!("\n══════════════════════════════════════════════════════════\n");
}

