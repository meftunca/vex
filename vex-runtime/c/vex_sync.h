/**
 * vex_sync.h - Concurrency Primitives
 * 
 * Thread-safe synchronization types for Vex stdlib/sync module:
 * - Arc<T> - Atomic reference counting (thread-safe)
 * - Rc<T> - Reference counting (single-threaded)
 * - Mutex<T> - Mutual exclusion lock
 * - RwLock<T> - Read-write lock
 * - Atomic<T> - Lock-free atomics (i32, i64, u32, u64, bool, ptr)
 * - Barrier - Thread synchronization barrier
 * - Once - One-time initialization
 * 
 * Platform Support:
 * - POSIX: pthread_mutex, pthread_rwlock
 * - Linux: pthread_barrier (native)
 * - macOS: Custom barrier implementation (pthread_barrier not available)
 * - C11: stdatomic.h (atomic_int, atomic_uint_fast64_t, etc.)
 * - Lock-free: All Atomic<T> operations use compiler intrinsics
 * 
 * Performance Goals:
 * - Arc/Rc: Single allocation (metadata + value inline)
 * - Mutex: Zero-overhead wrapper over pthread_mutex
 * - Atomic: Direct CPU instructions (no syscalls)
 * - Cache-aligned: 64-byte alignment for shared data
 * 
 * Safety:
 * - Panic on lock poisoning (mutex used after panic)
 * - Panic on use-after-free (refcount checks)
 * - Deadlock detection (debug builds only)
 * 
 * Date: November 18, 2025
 */

#ifndef VEX_SYNC_H
#define VEX_SYNC_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdatomic.h>
#include <pthread.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// ATOMIC REFERENCE COUNTING - Arc<T>
// ============================================================================

/**
 * Arc<T> - Atomic Reference Counted pointer (thread-safe)
 * 
 * Memory Layout (single allocation):
 * ┌─────────────────────────────────────────┐
 * │ ArcInner:                               │
 * │   - atomic_size_t strong_count          │ 8 bytes (aligned)
 * │   - atomic_size_t weak_count            │ 8 bytes
 * │   - size_t data_size                    │ 8 bytes
 * │   - uint8_t data[...]                   │ N bytes (inline)
 * └─────────────────────────────────────────┘
 * 
 * Properties:
 * - Thread-safe: Uses atomic operations (no locks)
 * - Cache-friendly: Data stored inline (no pointer chasing)
 * - Drop semantics: Automatically freed when strong_count reaches 0
 * - Weak pointers: Support for breaking reference cycles
 */
typedef struct {
    void *inner;  // Opaque pointer to ArcInner
} vex_arc_t;

typedef struct {
    _Atomic(size_t) strong_count;  // Strong references
    _Atomic(size_t) weak_count;    // Weak references
    size_t data_size;              // Size of T in bytes
    uint8_t data[];                // Flexible array member (inline data)
} vex_arc_inner_t;

/**
 * Create new Arc<T> with initial value
 * Thread-safe: Yes (atomic initialization)
 * 
 * @param value Pointer to value to copy
 * @param size Size of T in bytes
 * @return Arc handle (never NULL, panics on OOM)
 */
vex_arc_t vex_arc_new(const void *value, size_t size);

/**
 * Clone Arc (increment strong count)
 * Thread-safe: Yes (atomic increment)
 * 
 * @param arc Arc to clone
 * @return New Arc pointing to same data
 */
vex_arc_t vex_arc_clone(vex_arc_t arc);

/**
 * Get immutable reference to inner value
 * Thread-safe: Yes (read-only access)
 * 
 * @param arc Arc to dereference
 * @return Pointer to T (do not free!)
 */
const void *vex_arc_get(const vex_arc_t *arc);

/**
 * Get strong reference count
 * Thread-safe: Yes (atomic load)
 * 
 * @param arc Arc to query
 * @return Current strong count
 */
size_t vex_arc_strong_count(const vex_arc_t *arc);

/**
 * Drop Arc (decrement strong count, free if 0)
 * Thread-safe: Yes (atomic decrement + memory barrier)
 * 
 * @param arc Arc to drop
 */
void vex_arc_drop(vex_arc_t arc);

/**
 * Try to get mutable reference (only if strong_count == 1)
 * Thread-safe: Yes (atomic check)
 * 
 * @param arc Arc to get mut ref
 * @return Mutable pointer if unique, NULL otherwise
 */
void *vex_arc_get_mut(vex_arc_t *arc);

// ============================================================================
// REFERENCE COUNTING - Rc<T>
// ============================================================================

/**
 * Rc<T> - Reference Counted pointer (single-threaded)
 * 
 * Memory Layout (same as Arc, but non-atomic):
 * ┌─────────────────────────────────────────┐
 * │ RcInner:                                │
 * │   - size_t strong_count                 │ 8 bytes
 * │   - size_t weak_count                   │ 8 bytes
 * │   - size_t data_size                    │ 8 bytes
 * │   - uint8_t data[...]                   │ N bytes (inline)
 * └─────────────────────────────────────────┘
 * 
 * Faster than Arc (no atomic overhead), but NOT thread-safe!
 */
typedef struct {
    void *inner;  // Opaque pointer to RcInner
} vex_rc_t;

typedef struct {
    size_t strong_count;  // Non-atomic
    size_t weak_count;    // Non-atomic
    size_t data_size;
    uint8_t data[];
} vex_rc_inner_t;

vex_rc_t vex_rc_new(const void *value, size_t size);
vex_rc_t vex_rc_clone(vex_rc_t rc);
const void *vex_rc_get(const vex_rc_t *rc);
void *vex_rc_get_mut(vex_rc_t *rc);
size_t vex_rc_strong_count(const vex_rc_t *rc);
void vex_rc_drop(vex_rc_t rc);

// ============================================================================
// MUTEX - Mutual Exclusion Lock
// ============================================================================

/**
 * Mutex<T> - Mutual exclusion lock with inner value
 * 
 * Memory Layout (single allocation):
 * ┌─────────────────────────────────────────┐
 * │ MutexInner:                             │
 * │   - pthread_mutex_t lock                │ 40+ bytes (platform-specific)
 * │   - bool poisoned                       │ 1 byte
 * │   - size_t data_size                    │ 8 bytes
 * │   - uint8_t data[...]                   │ N bytes (inline)
 * └─────────────────────────────────────────┘
 * 
 * Properties:
 * - RAII: MutexGuard unlocks on drop
 * - Poisoning: Panic if locked thread panics
 * - Recursive: NOT supported (will deadlock)
 */
typedef struct {
    void *inner;  // Opaque pointer to MutexInner
} vex_mutex_t;

typedef struct {
    pthread_mutex_t lock;
    bool poisoned;
    size_t data_size;
    uint8_t data[];
} vex_mutex_inner_t;

/**
 * MutexGuard - RAII lock guard (auto-unlock on drop)
 */
typedef struct {
    vex_mutex_t *mutex;  // Borrowed mutex
    void *data;          // Pointer to locked data
} vex_mutex_guard_t;

vex_mutex_t vex_mutex_new(const void *value, size_t size);
vex_mutex_guard_t vex_mutex_lock(vex_mutex_t *mutex);
bool vex_mutex_try_lock(vex_mutex_t *mutex, vex_mutex_guard_t *out_guard);
void vex_mutex_guard_drop(vex_mutex_guard_t guard);
void vex_mutex_drop(vex_mutex_t mutex);

/**
 * Get mutable reference from guard (safe - we hold the lock)
 */
void *vex_mutex_guard_get_mut(vex_mutex_guard_t *guard);

// ============================================================================
// RWLOCK - Read-Write Lock
// ============================================================================

/**
 * RwLock<T> - Read-Write lock (multiple readers, single writer)
 * 
 * Properties:
 * - Many readers: Multiple threads can read simultaneously
 * - Exclusive writer: Only one writer, blocks readers
 * - Fair: Writer-preferring (readers don't starve writers)
 */
typedef struct {
    void *inner;
} vex_rwlock_t;

typedef struct {
    pthread_rwlock_t lock;
    bool poisoned;
    size_t data_size;
    uint8_t data[];
} vex_rwlock_inner_t;

typedef struct {
    vex_rwlock_t *lock;
    void *data;
    bool is_write;  // true = write lock, false = read lock
} vex_rwlock_guard_t;

vex_rwlock_t vex_rwlock_new(const void *value, size_t size);
vex_rwlock_guard_t vex_rwlock_read(vex_rwlock_t *lock);
vex_rwlock_guard_t vex_rwlock_write(vex_rwlock_t *lock);
bool vex_rwlock_try_read(vex_rwlock_t *lock, vex_rwlock_guard_t *out_guard);
bool vex_rwlock_try_write(vex_rwlock_t *lock, vex_rwlock_guard_t *out_guard);
void vex_rwlock_guard_drop(vex_rwlock_guard_t guard);
void vex_rwlock_drop(vex_rwlock_t lock);

const void *vex_rwlock_guard_get(const vex_rwlock_guard_t *guard);
void *vex_rwlock_guard_get_mut(vex_rwlock_guard_t *guard);

// ============================================================================
// ATOMIC - Lock-Free Atomics
// ============================================================================

/**
 * Atomic<T> - Lock-free atomic operations
 * 
 * Supported Types:
 * - i32, i64 (atomic_int_fast32_t, atomic_int_fast64_t)
 * - u32, u64 (atomic_uint_fast32_t, atomic_uint_fast64_t)
 * - bool (atomic_bool)
 * - ptr (atomic_uintptr_t)
 * 
 * Memory Ordering (Rust-compatible):
 * - Relaxed: No synchronization (fastest)
 * - Acquire: Synchronize loads (read-only ops)
 * - Release: Synchronize stores (write-only ops)
 * - AcqRel: Synchronize both (read-modify-write)
 * - SeqCst: Sequential consistency (slowest, safest)
 */

typedef enum {
    VEX_ORDERING_RELAXED = memory_order_relaxed,
    VEX_ORDERING_ACQUIRE = memory_order_acquire,
    VEX_ORDERING_RELEASE = memory_order_release,
    VEX_ORDERING_ACQ_REL = memory_order_acq_rel,
    VEX_ORDERING_SEQ_CST = memory_order_seq_cst,
} vex_atomic_ordering_t;

// Atomic i32
typedef _Atomic(int32_t) vex_atomic_i32_t;

int32_t vex_atomic_i32_load(const vex_atomic_i32_t *atomic, vex_atomic_ordering_t order);
void vex_atomic_i32_store(vex_atomic_i32_t *atomic, int32_t value, vex_atomic_ordering_t order);
int32_t vex_atomic_i32_swap(vex_atomic_i32_t *atomic, int32_t value, vex_atomic_ordering_t order);
bool vex_atomic_i32_compare_exchange(vex_atomic_i32_t *atomic, int32_t *expected, int32_t desired, vex_atomic_ordering_t order);
int32_t vex_atomic_i32_fetch_add(vex_atomic_i32_t *atomic, int32_t value, vex_atomic_ordering_t order);
int32_t vex_atomic_i32_fetch_sub(vex_atomic_i32_t *atomic, int32_t value, vex_atomic_ordering_t order);

// Atomic i64
typedef _Atomic(int64_t) vex_atomic_i64_t;

int64_t vex_atomic_i64_load(const vex_atomic_i64_t *atomic, vex_atomic_ordering_t order);
void vex_atomic_i64_store(vex_atomic_i64_t *atomic, int64_t value, vex_atomic_ordering_t order);
int64_t vex_atomic_i64_swap(vex_atomic_i64_t *atomic, int64_t value, vex_atomic_ordering_t order);
bool vex_atomic_i64_compare_exchange(vex_atomic_i64_t *atomic, int64_t *expected, int64_t desired, vex_atomic_ordering_t order);
int64_t vex_atomic_i64_fetch_add(vex_atomic_i64_t *atomic, int64_t value, vex_atomic_ordering_t order);
int64_t vex_atomic_i64_fetch_sub(vex_atomic_i64_t *atomic, int64_t value, vex_atomic_ordering_t order);

// Atomic u32
typedef _Atomic(uint32_t) vex_atomic_u32_t;

uint32_t vex_atomic_u32_load(const vex_atomic_u32_t *atomic, vex_atomic_ordering_t order);
void vex_atomic_u32_store(vex_atomic_u32_t *atomic, uint32_t value, vex_atomic_ordering_t order);
uint32_t vex_atomic_u32_swap(vex_atomic_u32_t *atomic, uint32_t value, vex_atomic_ordering_t order);
bool vex_atomic_u32_compare_exchange(vex_atomic_u32_t *atomic, uint32_t *expected, uint32_t desired, vex_atomic_ordering_t order);
uint32_t vex_atomic_u32_fetch_add(vex_atomic_u32_t *atomic, uint32_t value, vex_atomic_ordering_t order);
uint32_t vex_atomic_u32_fetch_sub(vex_atomic_u32_t *atomic, uint32_t value, vex_atomic_ordering_t order);

// Atomic u64
typedef _Atomic(uint64_t) vex_atomic_u64_t;

uint64_t vex_atomic_u64_load(const vex_atomic_u64_t *atomic, vex_atomic_ordering_t order);
void vex_atomic_u64_store(vex_atomic_u64_t *atomic, uint64_t value, vex_atomic_ordering_t order);
uint64_t vex_atomic_u64_swap(vex_atomic_u64_t *atomic, uint64_t value, vex_atomic_ordering_t order);
bool vex_atomic_u64_compare_exchange(vex_atomic_u64_t *atomic, uint64_t *expected, uint64_t desired, vex_atomic_ordering_t order);
uint64_t vex_atomic_u64_fetch_add(vex_atomic_u64_t *atomic, uint64_t value, vex_atomic_ordering_t order);
uint64_t vex_atomic_u64_fetch_sub(vex_atomic_u64_t *atomic, uint64_t value, vex_atomic_ordering_t order);

// Atomic bool
typedef _Atomic(bool) vex_atomic_bool_t;

bool vex_atomic_bool_load(const vex_atomic_bool_t *atomic, vex_atomic_ordering_t order);
void vex_atomic_bool_store(vex_atomic_bool_t *atomic, bool value, vex_atomic_ordering_t order);
bool vex_atomic_bool_swap(vex_atomic_bool_t *atomic, bool value, vex_atomic_ordering_t order);
bool vex_atomic_bool_compare_exchange(vex_atomic_bool_t *atomic, bool *expected, bool desired, vex_atomic_ordering_t order);

// Atomic ptr
typedef _Atomic(uintptr_t) vex_atomic_ptr_t;

void *vex_atomic_ptr_load(const vex_atomic_ptr_t *atomic, vex_atomic_ordering_t order);
void vex_atomic_ptr_store(vex_atomic_ptr_t *atomic, void *value, vex_atomic_ordering_t order);
void *vex_atomic_ptr_swap(vex_atomic_ptr_t *atomic, void *value, vex_atomic_ordering_t order);
bool vex_atomic_ptr_compare_exchange(vex_atomic_ptr_t *atomic, void **expected, void *desired, vex_atomic_ordering_t order);

// ============================================================================
// BARRIER - Thread Synchronization Barrier
// ============================================================================

/**
 * Barrier - Thread Synchronization Point
 * 
 * Blocks threads until N threads reach the barrier.
 * Last thread to arrive unblocks all threads.
 * 
 * Usage:
 *   Barrier *b = vex_barrier_new(3);  // Wait for 3 threads
 *   vex_barrier_wait(b);              // Blocks until 3 threads call this
 */
#ifdef __APPLE__
/* macOS doesn't have pthread_barrier_t, use custom implementation */
typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    size_t count;
    size_t current;
    size_t generation;
} vex_barrier_t;
#else
/* Linux and other POSIX systems have pthread_barrier_t */
typedef struct {
    pthread_barrier_t barrier;
    size_t count;
} vex_barrier_t;
#endif

vex_barrier_t *vex_barrier_new(size_t count);
void vex_barrier_wait(vex_barrier_t *barrier);
void vex_barrier_drop(vex_barrier_t *barrier);

// ============================================================================
// ONCE - One-Time Initialization
// ============================================================================

/**
 * Once - Execute function exactly once (thread-safe)
 * 
 * Usage:
 *   static vex_once_t INIT = VEX_ONCE_INIT;
 *   vex_once_call(&INIT, init_function, arg);
 */
typedef struct {
    _Atomic(uint32_t) state;  // 0=uninitialized, 1=running, 2=done
    pthread_mutex_t lock;
} vex_once_t;

#define VEX_ONCE_INIT { ATOMIC_VAR_INIT(0), PTHREAD_MUTEX_INITIALIZER }

void vex_once_call(vex_once_t *once, void (*func)(void*), void *arg);

// ============================================================================
// CONDVAR - Condition Variable
// ============================================================================

/**
 * Condvar - Condition variable for waiting on complex conditions
 * 
 * Usage:
 *   vex_condvar_t *cv = vex_condvar_new();
 *   vex_condvar_wait(cv, &mutex_guard);  // Atomically unlock+wait
 *   vex_condvar_notify_one(cv);          // Wake one waiter
 *   vex_condvar_notify_all(cv);          // Wake all waiters
 */
typedef struct {
    pthread_cond_t cond;
} vex_condvar_t;

vex_condvar_t *vex_condvar_new(void);
void vex_condvar_wait(vex_condvar_t *cv, vex_mutex_guard_t *guard);
bool vex_condvar_wait_timeout(vex_condvar_t *cv, vex_mutex_guard_t *guard, uint64_t timeout_ms);
void vex_condvar_notify_one(vex_condvar_t *cv);
void vex_condvar_notify_all(vex_condvar_t *cv);
void vex_condvar_drop(vex_condvar_t *cv);

// ============================================================================
// THREAD UTILITIES
// ============================================================================

/**
 * Memory fence (compiler + CPU barrier)
 */
void vex_fence(vex_atomic_ordering_t order);

/**
 * Spin-wait hint (CPU optimization)
 */
void vex_spin_loop_hint(void);

#ifdef __cplusplus
}
#endif

#endif // VEX_SYNC_H
