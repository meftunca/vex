# Performance Benchmark Suite

## Summary

### Phase 1: Baseline
- **Before Optimizations**: ~754 msg/s

### Phase 2: Task Pool + Batch Processing  
- **After Optimizations**: ~927 msg/s
- **Improvement**: +23% (173 msg/s gain)

## Optimizations Applied

1. ✅ **Task Object Pool** - Zero-allocation recycling
2. ✅ **Batch Event Processing** - Reduced queue contention
3. ✅ **Optimized Poller** - Single syscall per operation

## Gap Analysis: 927 msg/s → 1M msg/s

**Gap**: ~1,079x improvement still needed

### Remaining Bottlenecks

The current architecture is fundamentally limited by:

1. **Network I/O overhead** (~1ms per socket operation)
2. **Syscall latency** (k queue ~500ns-1μs per call)
3. **Scheduler overhead** (context switch ~1-5μs)

### To Reach 1M msg/s

The current approach processes one message per task. To achieve 1M msg/s:

**Option A: Synthetic Benchmark** (Pure scheduling)
- Remove real I/O from benchmark
- Test pure task scheduling throughput
- Expected: 100K-500K msg/s

**Option B: Architectural Change** (Batched I/O)
- Process multiple messages per I/O operation
- Use message batching (like io_uring batching)
- Buffer and batch socket reads/writes

**Option C: Realistic Target Adjustment**
- Network I/O fundamentally limits throughput
- Focus on latency optimization instead
- Target: <10μs scheduling overhead per message

## Next Steps

1. Create synthetic benchmark (no real I/O)
2. Measure pure scheduling overhead 
3. If <10μs per task → architecture is good
4. If >10μs → need deeper optimization

## Benchmark Results

```
test_stress_10k:
- Messages: 9275/10000 (92%)
- Time: 10 seconds
- Throughput: 927 msg/s
- Latency: ~10.8ms per message
```

**Bottleneck**: Network socket I/O dominates (most time spent waiting for sockets, not scheduling)
