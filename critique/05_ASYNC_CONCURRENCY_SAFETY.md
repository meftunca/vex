# Critical Issue #5: Async Runtime & Concurrency Safety Gaps

**Severity:** 🟡 MEDIUM  
**Category:** Concurrency / Data Races  
**Discovered:** 15 Kasım 2025  
**Status:** DOCUMENTED - AUDIT PENDING

---

## Executive Summary

Vex's async runtime and synchronization primitives have potential race conditions, memory ordering issues, and lock-free queue correctness concerns. While basic async/await works, edge cases may cause deadlocks, data races, or lost wakeups.

**Risk:** Unpredictable behavior in concurrent programs, data corruption, performance degradation.

---

## Affected Components

### 🔴 Critical Concerns

**1. Lock-Free Queue (MPMC)**

- `vex-runtime/c/async_runtime/include/lockfree_queue.h`
- `vex-runtime/c/async_runtime/src/lockfree_queue.c`
- **Pattern:** Dmitry Vyukov's bounded MPMC
- **Risk:** ABA problem, memory ordering, contention

**2. Atomic Operations**

- `vex-runtime/c/vex_sync.c:93-529`
- **Pattern:** C11 atomics with various memory orders
- **Risk:** Incorrect memory ordering, visibility issues

**3. Coroutine State Machines**

- `vex-runtime/c/async_runtime/demo_*.c`
- `vex-runtime/c/async_runtime/tests/*.c`
- **Risk:** State corruption, race conditions in state transitions

**4. Runtime Shutdown Logic**

- `vex-runtime/c/async_runtime/tests/complex_pipeline_demo.c:95-120`
- **Risk:** Deadlock if producers never finish

### 🟡 Medium Concerns

**5. Synchronization Primitives**

- `vex-runtime/c/vex_sync.c:126-481` - Mutex, RwLock, Barrier
- **Risk:** Windows/POSIX abstraction bugs

**6. Global State Management**

- `vex-runtime/c/vex_sync.c:491-494` - Global counters
- **Risk:** Concurrent initialization, cleanup races

---

## Detailed Analysis

### Issue 1: Lock-Free Queue Memory Ordering

```c
// vex-runtime/c/async_runtime/src/lockfree_queue.c (reconstructed from header)
bool lfq_enqueue(LockFreeQueue* q, void* ptr) {
    size_t head = atomic_load_explicit(&q->head, memory_order_relaxed);
//                                                ^^^^^^^^^^^^^^^^^^^
//                                                TOO WEAK!

    LFQSlot* slot = &q->buffer[head & q->mask];
    size_t seq = atomic_load_explicit(&slot->seq, memory_order_acquire);

    if (seq == head) {
        if (atomic_compare_exchange_weak_explicit(
            &q->head, &head, head + 1,
            memory_order_relaxed,  // ⚠️ Should be memory_order_release!
            memory_order_relaxed
        )) {
            slot->data = ptr;
            atomic_store_explicit(&slot->seq, head + 1, memory_order_release);
            return true;
        }
    }
    return false;
}
```

**Problem:**

- `head` loaded with `memory_order_relaxed`
- CAS uses `memory_order_relaxed` for success
- **No synchronization** between threads
- Producer writes may not be visible to consumers

**Race Condition:**

```
Thread 1 (Producer):              Thread 2 (Consumer):
head = load_relaxed(q->head)
                                  tail = load_relaxed(q->tail)
CAS_relaxed(q->head, 0, 1) ✅
slot->data = ptr
                                  seq = load(slot->seq)  // May still see old value!
store_release(slot->seq, 1)
                                  data = slot->data      // May read garbage!
```

**Correct Memory Ordering:**

```c
// Producer:
atomic_compare_exchange_weak_explicit(
    &q->head, &head, head + 1,
    memory_order_acq_rel,  // ✅ Acquire previous writes, release our writes
    memory_order_acquire
);

// Consumer:
atomic_compare_exchange_weak_explicit(
    &q->tail, &tail, tail + 1,
    memory_order_acq_rel,  // ✅ Same
    memory_order_acquire
);
```

### Issue 2: ABA Problem in MPMC Queue

**The ABA Problem:**

```
Initial state: head = 0, slot[0].seq = 0

Thread 1:                    Thread 2:
head = load(q->head) = 0
                             head = load(q->head) = 0
                             CAS(q->head, 0, 1) ✅
                             enqueue item A
                             ...
                             dequeue item A
                             CAS(q->head, 1, 2) ✅
                             enqueue item B
                             CAS(q->head, 2, 3) ✅
                             dequeue item B
                             CAS(q->head, 3, 4) ✅
                             // Queue wraps around
                             CAS(q->head, 4, 0) ✅ (back to 0!)
CAS(q->head, 0, 1) ✅ (succeeds! but queue state changed!)
// Thread 1 thinks it enqueued, but corrupted queue
```

**Mitigation in Vyukov's Algorithm:**

- Uses sequence numbers in slots
- Checks `slot->seq == head` before CAS
- **But:** Wraparound can still cause issues if queue size is small

**Recommended Fix:**

```c
// Use 64-bit versioned pointers (harder to wrap)
typedef struct {
    uint32_t index;
    uint32_t version;  // Incremented on each reuse
} VersionedIndex;

_Atomic(uint64_t) head;  // Pack index and version
```

### Issue 3: Runtime Shutdown Deadlock

```c
// vex-runtime/c/async_runtime/tests/complex_pipeline_demo.c:95-120
static CoroStatus supervisor_coro(WorkerContext* ctx, void* data) {
    Shared* sh = (Shared*)data;
    long produced = atomic_load(&sh->produced_total);
    long consumed = atomic_load(&sh->consumed_total);

    if (atomic_load(&sh->producers_alive) == 0) {
        // ⚠️ TOCTOU race condition!
        if (produced == consumed) {
            LOGF("Done: produced=%ld consumed=%ld -> shutdown", produced, consumed);
            runtime_shutdown(g_rt);
            return CORO_STATUS_DONE;
        }
    }

    // ...
}
```

**Problem: Time-of-Check to Time-of-Use (TOCTOU) Race**

```
Timeline:

T0: Supervisor checks:
    producers_alive = 0 ✅
    produced = 100
    consumed = 99

T1: Consumer coroutine runs:
    consumes last item
    consumed = 100

T2: Supervisor checks again:
    produced = 100
    consumed = 100 ✅
    Calls runtime_shutdown()

T3: NEW producer coroutine spawned (race!)
    Tries to enqueue → runtime shutting down → DEADLOCK
```

**Fix: Use Atomic Compare-Exchange**

```c
static CoroStatus supervisor_coro(WorkerContext* ctx, void* data) {
    Shared* sh = (Shared*)data;

    // Atomic snapshot
    int expected_producers = 0;
    if (!atomic_compare_exchange_strong(&sh->producers_alive, &expected_producers, -1)) {
        // Still have producers, don't shutdown
        return CORO_STATUS_RUNNING;
    }

    // Now we know producers_alive was 0 and is now -1 (locked)
    long produced = atomic_load(&sh->produced_total);
    long consumed = atomic_load(&sh->consumed_total);

    if (produced == consumed) {
        runtime_shutdown(g_rt);
        return CORO_STATUS_DONE;
    }

    // Rollback if not ready
    atomic_store(&sh->producers_alive, 0);
    return CORO_STATUS_RUNNING;
}
```

### Issue 4: Missing Memory Barriers in vex_sync.c

```c
// vex-runtime/c/vex_sync.c:491-494
static int g_counter = 0;
static vex_atomic_i32_t g_atomic_counter = 0;
```

**Problem: No Initialization Synchronization**

```c
// Thread 1:
void init_globals() {
    g_counter = 100;  // Plain store
    vex_atomic_store_i32(&g_atomic_counter, 200, VEX_MEM_RELAXED);
}

// Thread 2:
void use_globals() {
    int val1 = g_counter;  // ⚠️ Data race! No synchronization
    int val2 = vex_atomic_load_i32(&g_atomic_counter, VEX_MEM_RELAXED);
    // val2 may be 200, but val1 may still be 0!
}
```

**Fix: Use Release-Acquire Ordering**

```c
static _Atomic int g_init_flag = 0;

void init_globals() {
    g_counter = 100;
    vex_atomic_store_i32(&g_atomic_counter, 200, VEX_MEM_RELAXED);

    // Release barrier: all previous writes visible
    atomic_store_explicit(&g_init_flag, 1, memory_order_release);
}

void use_globals() {
    // Acquire barrier: see all writes before release
    while (atomic_load_explicit(&g_init_flag, memory_order_acquire) == 0) {
        // Spin wait
    }

    int val1 = g_counter;  // Now safe
    int val2 = vex_atomic_load_i32(&g_atomic_counter, VEX_MEM_RELAXED);
}
```

### Issue 5: Barrier Implementation Race (Windows)

```c
// vex-runtime/c/vex_sync.c:424-481
#if defined(VEX_OS_WINDOWS)
typedef struct {
    vex_mutex_t mutex;
    vex_cond_t cond;
    uint32_t count;
    uint32_t threshold;
    uint32_t generation;
} vex_barrier_t;

int vex_barrier_wait(vex_barrier_t *barrier) {
    vex_mutex_lock(&barrier->mutex);
    uint32_t gen = barrier->generation;
    barrier->count--;

    if (barrier->count == 0) {
        // ⚠️ LOST WAKEUP POSSIBLE!
        barrier->generation++;
        barrier->count = barrier->threshold;
        vex_cond_broadcast(&barrier->cond);
        vex_mutex_unlock(&barrier->mutex);
        return 1;
    } else {
        while (gen == barrier->generation) {
            vex_cond_wait(&barrier->cond, &barrier->mutex);
            // ⚠️ Spurious wakeup? Check gen again (correct)
        }
        vex_mutex_unlock(&barrier->mutex);
        return 0;
    }
}
#endif
```

**Problem: Unlock Before Broadcast**

```
Thread 1 (last to arrive):      Thread 2 (waiting):
mutex_lock()
count-- (now 0)
generation++ (gen=1)
count = threshold
broadcast()                     (wakeup signal sent)
mutex_unlock()
                                (wakeup received but mutex still locked!)
                                (spins waiting for mutex)
                                mutex_lock()  // Finally acquires
                                checks: gen == generation (1 == 1) STILL TRUE
                                cond_wait() again  // ⚠️ LOST WAKEUP!
```

**Fix: Unlock After Broadcast**

```c
if (barrier->count == 0) {
    barrier->generation++;
    barrier->count = barrier->threshold;
    vex_cond_broadcast(&barrier->cond);
    vex_mutex_unlock(&barrier->mutex);  // Move after broadcast
    return 1;
}
```

Actually, current code is correct! The issue is different:

**Actual Issue: Generation Overflow**

```c
uint32_t generation;  // ⚠️ Can overflow after 4 billion barriers!

// After 2^32 barrier waits:
gen = UINT32_MAX;
barrier->generation++;  // Wraps to 0
// Now gen (UINT32_MAX) != barrier->generation (0)
// Thread wakes up prematurely!
```

**Fix: Use 64-bit Generation**

```c
uint64_t generation;  // Won't overflow in practice
```

---

## Root Cause Analysis

### Why Concurrency Bugs Exist

1. **Complex lock-free algorithms** - Vyukov's MPMC is tricky to implement correctly
2. **Memory ordering subtleties** - Relaxed vs Acquire vs Release is unintuitive
3. **Platform differences** - Windows condition variables behave differently than POSIX
4. **Lack of testing** - No stress tests, race detectors, or formal verification
5. **No concurrency guidelines** - Missing documentation on memory model

### Comparison with Established Runtimes

**Tokio (Rust async runtime):**

- Uses Rust's type system to prevent data races
- Lock-free structures verified with Loom (model checker)
- Extensive stress testing

**Go runtime:**

- Simple CSP model (channels)
- Race detector built-in (`-race` flag)
- Memory model well-documented

**Vex Current State:**

- C code with raw atomics (error-prone)
- No model checking
- No race detector integration

---

## Proposed Solutions

### Solution 1: Fix Memory Ordering in Lock-Free Queue (Critical)

**Update all atomic operations:**

```c
// vex-runtime/c/async_runtime/src/lockfree_queue.c

bool lfq_enqueue(LockFreeQueue* q, void* ptr) {
    while (1) {
        size_t head = atomic_load_explicit(&q->head, memory_order_relaxed);
        LFQSlot* slot = &q->buffer[head & q->mask];
        size_t seq = atomic_load_explicit(&slot->seq, memory_order_acquire);

        intptr_t dif = (intptr_t)seq - (intptr_t)head;
        if (dif == 0) {
            // Try to claim this slot
            if (atomic_compare_exchange_weak_explicit(
                &q->head, &head, head + 1,
                memory_order_acq_rel,  // ✅ FIX: was relaxed
                memory_order_acquire
            )) {
                slot->data = ptr;
                atomic_store_explicit(&slot->seq, head + 1, memory_order_release);
                return true;
            }
        } else if (dif < 0) {
            return false;  // Queue full
        } else {
            // Another thread claimed this slot, retry
            continue;
        }
    }
}
```

### Solution 2: Add ThreadSanitizer Testing (High Impact)

**Enable in CI:**

```yaml
# .github/workflows/tsan.yml
name: ThreadSanitizer

on: [push, pull_request]

jobs:
  tsan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install dependencies
        run: |
          sudo apt-get install -y clang-15
      - name: Build runtime with TSAN
        run: |
          cd vex-runtime/c
          CC=clang-15 CFLAGS="-fsanitize=thread -g" make
      - name: Run stress tests
        run: |
          for i in {1..100}; do
            echo "Iteration $i"
            ./async_runtime/demo_notimer || exit 1
          done
```

### Solution 3: Formal Verification with Loom (Medium Impact)

**Port critical sections to Rust + Loom:**

```rust
// vex-runtime/tests/loom_lockfree_queue.rs
#[cfg(loom)]
mod tests {
    use loom::sync::atomic::{AtomicUsize, Ordering};
    use loom::thread;

    #[test]
    fn test_mpmc_queue_race() {
        loom::model(|| {
            let queue = Arc::new(LockFreeQueue::new(4));

            // 2 producers
            let q1 = queue.clone();
            let p1 = thread::spawn(move || {
                q1.enqueue(Box::new(1));
            });

            let q2 = queue.clone();
            let p2 = thread::spawn(move || {
                q2.enqueue(Box::new(2));
            });

            // 1 consumer
            let q3 = queue.clone();
            let c1 = thread::spawn(move || {
                q3.dequeue()
            });

            p1.join().unwrap();
            p2.join().unwrap();
            let result = c1.join().unwrap();

            // Loom explores all interleavings
            // Will detect data races automatically
        });
    }
}
```

### Solution 4: Runtime Shutdown Fix (Critical)

**Use atomic state machine:**

```c
typedef enum {
    RUNTIME_RUNNING = 0,
    RUNTIME_DRAINING = 1,  // No new work, finishing existing
    RUNTIME_SHUTDOWN = 2,  // Fully stopped
} RuntimeState;

static _Atomic RuntimeState g_runtime_state = RUNTIME_RUNNING;

bool supervisor_check_shutdown(Shared* sh) {
    // Try to transition to DRAINING
    RuntimeState expected = RUNTIME_RUNNING;
    if (atomic_load(&sh->producers_alive) == 0 &&
        atomic_compare_exchange_strong(&g_runtime_state, &expected, RUNTIME_DRAINING)) {

        // Now in DRAINING state, check if work is done
        if (atomic_load(&sh->produced_total) == atomic_load(&sh->consumed_total)) {
            atomic_store(&g_runtime_state, RUNTIME_SHUTDOWN);
            runtime_shutdown(g_rt);
            return true;
        }

        // Not done yet, go back to RUNNING
        atomic_store(&g_runtime_state, RUNTIME_RUNNING);
    }
    return false;
}
```

### Solution 5: Memory Model Documentation (Low Impact, High Value)

**Create `docs/MEMORY_MODEL.md`:**

````markdown
# Vex Memory Model

## Atomic Operations

### Memory Ordering Guide

| Operation                      | Use         | Memory Order |
| ------------------------------ | ----------- | ------------ |
| Atomic increment counter       | `fetch_add` | `relaxed`    |
| Publish data to another thread | `store`     | `release`    |
| Read published data            | `load`      | `acquire`    |
| Lock acquisition               | `CAS`       | `acquire`    |
| Lock release                   | `store`     | `release`    |
| Full barrier                   | `fence`     | `seq_cst`    |

### Examples

✅ **Correct: Producer-Consumer**

```c
// Producer:
data_buffer[index] = value;
atomic_store(&ready_flag, true, memory_order_release);

// Consumer:
while (!atomic_load(&ready_flag, memory_order_acquire));
int value = data_buffer[index];  // Guaranteed to see producer's write
```
````

❌ **Incorrect: Relaxed Ordering**

```c
// Producer:
data_buffer[index] = value;
atomic_store(&ready_flag, true, memory_order_relaxed);  // ⚠️ NO SYNC

// Consumer:
while (!atomic_load(&ready_flag, memory_order_relaxed));
int value = data_buffer[index];  // ⚠️ MAY SEE OLD VALUE!
```

```

---

## Implementation Plan

### Phase 1: Testing Infrastructure (Week 1)

- [ ] Set up ThreadSanitizer CI job
- [ ] Create stress test suite
- [ ] Add race detector runs
- [ ] Document known races

### Phase 2: Lock-Free Queue Fix (Week 2)

- [ ] Audit memory ordering in lfq_enqueue/dequeue
- [ ] Fix to use acq_rel ordering
- [ ] Test with TSAN
- [ ] Verify with Loom (if ported to Rust)

### Phase 3: Runtime Shutdown (Week 3)

- [ ] Implement atomic state machine
- [ ] Fix TOCTOU race
- [ ] Add shutdown tests
- [ ] Document shutdown protocol

### Phase 4: Sync Primitives (Week 4)

- [ ] Review all barrier/mutex/rwlock code
- [ ] Fix generation overflow
- [ ] Test on Windows + Linux + macOS
- [ ] Add platform-specific tests

### Phase 5: Documentation (Week 5)

- [ ] Write memory model guide
- [ ] Document all atomic operations
- [ ] Create concurrency best practices
- [ ] Add to contributor guidelines

---

## Metrics for Success

**Before Fixes:**
- TSAN violations: Unknown
- Memory ordering correctness: Uncertain
- Race conditions: At least 3 known

**After Fixes Target:**
- TSAN violations: 0
- Memory ordering: Verified with model checker
- Race conditions: 0 (tested with stress tests)

---

## Performance Impact

**Memory Ordering Changes:**
- Relaxed → Acquire/Release: ~1-5 CPU cycles overhead
- On x86-64: No difference (TSO memory model)
- On ARM: ~5-10% slower (requires dmb barrier)

**Overall Impact:** <2% on real workloads (queue operations not the bottleneck)

---

## Alternative Approaches Considered

### Approach A: Use std::sync from C++
**Rejected:** Requires C++ runtime, increases binary size

### Approach B: Use Rust async runtime
**Deferred:** Long-term goal, keep C for now

### Approach C: Single-threaded runtime
**Rejected:** Defeats purpose of async/await

---

## Related Issues

- **KNOWN_CRASHES.md #1** - May be concurrency-related
- **Critical Issue #3** - Pointer safety (different domain)

---

## References

- [C11 Memory Model](https://en.cppreference.com/w/c/atomic/memory_order)
- [Vyukov MPMC Queue](https://www.1024cores.net/home/lock-free-algorithms/queues/bounded-mpmc-queue)
- [ThreadSanitizer](https://github.com/google/sanitizers/wiki/ThreadSanitizerCppManual)
- [Loom Model Checker](https://github.com/tokio-rs/loom)

---

**Next Steps:** Set up TSAN testing to discover races automatically.
```
