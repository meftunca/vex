package main

import (
	"fmt"
	"time"
)

func main() {
	// RFC3339 Parse Benchmark
	testStr := "2024-11-07T12:34:56.123456789Z"
	iterations := 1000000

	start := time.Now()
	for i := 0; i < iterations; i++ {
		time.Parse(time.RFC3339Nano, testStr)
	}
	elapsed := time.Since(start)
	nsPerOp := float64(elapsed.Nanoseconds()) / float64(iterations)

	fmt.Printf("══════════════════════════════════════════════════════════\n")
	fmt.Printf("  Go time.Parse() Benchmark\n")
	fmt.Printf("══════════════════════════════════════════════════════════\n\n")
	fmt.Printf("RFC3339 Parse: %.1f ns/op (%.1fM ops/s)\n\n", nsPerOp, 1000.0/nsPerOp)

	// RFC3339 Format Benchmark
	t, _ := time.Parse(time.RFC3339Nano, testStr)

	start = time.Now()
	for i := 0; i < iterations; i++ {
		t.Format(time.RFC3339Nano)
	}
	elapsed = time.Since(start)
	nsPerOp = float64(elapsed.Nanoseconds()) / float64(iterations)

	fmt.Printf("RFC3339 Format: %.1f ns/op (%.1fM ops/s)\n\n", nsPerOp, 1000.0/nsPerOp)

	// Layout Parse Benchmark (complex layout)
	testStr2 := "Thu, 07 Nov 2024 12:34:56 +0000"
	layout2 := time.RFC1123Z

	start = time.Now()
	for i := 0; i < iterations; i++ {
		time.Parse(layout2, testStr2)
	}
	elapsed = time.Since(start)
	nsPerOp = float64(elapsed.Nanoseconds()) / float64(iterations)

	fmt.Printf("Layout Parse (RFC1123Z): %.1f ns/op (%.1fM ops/s)\n\n", nsPerOp, 1000.0/nsPerOp)

	// Layout Format Benchmark
	t2, _ := time.Parse(layout2, testStr2)

	start = time.Now()
	for i := 0; i < iterations; i++ {
		t2.Format(layout2)
	}
	elapsed = time.Since(start)
	nsPerOp = float64(elapsed.Nanoseconds()) / float64(iterations)

	fmt.Printf("Layout Format (RFC1123Z): %.1f ns/op (%.1fM ops/s)\n", nsPerOp, 1000.0/nsPerOp)

	fmt.Printf("\n══════════════════════════════════════════════════════════\n\n")
}

