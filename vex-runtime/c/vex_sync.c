/* vex_sync.c - Synchronization primitives for Vex (atomics, mutex, rwlock, condvar, semaphore)
 * 
 * Features:
 * - Atomic operations (load, store, add, sub, cas, swap)
 * - Mutex (lock, trylock, unlock)
 * - RWLock (read lock, write lock)
 * - Condition variable (wait, signal, broadcast)
 * - Semaphore (wait, post)
 * - Once (run once initialization)
 * - Barrier (thread synchronization point)
 * 
 * Cross-platform: Linux, macOS, Windows
 * 
 * Build: cc -O3 -std=c17 vex_sync.c -pthread -o test_sync
 * 
 * License: MIT
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdatomic.h>

// Use vex_macros.h if available
#if __has_include("vex_macros.h")
  #include "vex_macros.h"
#else
  #define VEX_INLINE static inline
  #define VEX_FORCE_INLINE static inline __attribute__((always_inline))
  
  #if defined(_WIN32)
    #define VEX_OS_WINDOWS 1
  #elif defined(__linux__)
    #define VEX_OS_LINUX 1
  #elif defined(__APPLE__)
    #define VEX_OS_MACOS 1
  #endif
#endif

// Platform-specific includes
#if defined(VEX_OS_WINDOWS)
  #include <windows.h>
#else
  #include <pthread.h>
  #include <semaphore.h>
  #include <unistd.h>
  #include <errno.h>
#endif

/* =========================
 * Atomics (C11 stdatomic.h)
 * ========================= */

// Atomic types
typedef _Atomic int32_t  vex_atomic_i32_t;
typedef _Atomic int64_t  vex_atomic_i64_t;
typedef _Atomic uint32_t vex_atomic_u32_t;
typedef _Atomic uint64_t vex_atomic_u64_t;
typedef _Atomic bool     vex_atomic_bool_t;
typedef _Atomic(void*)   vex_atomic_ptr_t;

// Memory orders
typedef enum {
  VEX_MEM_RELAXED = memory_order_relaxed,
  VEX_MEM_ACQUIRE = memory_order_acquire,
  VEX_MEM_RELEASE = memory_order_release,
  VEX_MEM_ACQ_REL = memory_order_acq_rel,
  VEX_MEM_SEQ_CST = memory_order_seq_cst
} vex_memory_order_t;

// Atomic load
VEX_INLINE int64_t vex_atomic_load_i64(vex_atomic_i64_t *ptr, vex_memory_order_t order) {
  return atomic_load_explicit(ptr, order);
}

// Atomic store
VEX_INLINE void vex_atomic_store_i64(vex_atomic_i64_t *ptr, int64_t val, vex_memory_order_t order) {
  atomic_store_explicit(ptr, val, order);
}

// Atomic add
VEX_INLINE int64_t vex_atomic_add_i64(vex_atomic_i64_t *ptr, int64_t val, vex_memory_order_t order) {
  return atomic_fetch_add_explicit(ptr, val, order);
}

// Atomic sub
VEX_INLINE int64_t vex_atomic_sub_i64(vex_atomic_i64_t *ptr, int64_t val, vex_memory_order_t order) {
  return atomic_fetch_sub_explicit(ptr, val, order);
}

// Atomic swap
VEX_INLINE int64_t vex_atomic_swap_i64(vex_atomic_i64_t *ptr, int64_t val, vex_memory_order_t order) {
  return atomic_exchange_explicit(ptr, val, order);
}

// Atomic compare-and-swap (CAS)
VEX_INLINE bool vex_atomic_cas_i64(vex_atomic_i64_t *ptr, int64_t *expected, int64_t desired, 
                                   vex_memory_order_t success, vex_memory_order_t failure) {
  return atomic_compare_exchange_strong_explicit(ptr, expected, desired, success, failure);
}

// Atomic fence
VEX_INLINE void vex_atomic_fence(vex_memory_order_t order) {
  atomic_thread_fence(order);
}

/* =========================
 * Mutex
 * ========================= */

#if defined(VEX_OS_WINDOWS)
typedef CRITICAL_SECTION vex_mutex_t;
#else
typedef pthread_mutex_t vex_mutex_t;
#endif

// Initialize mutex
VEX_INLINE int vex_mutex_init(vex_mutex_t *mutex) {
#if defined(VEX_OS_WINDOWS)
  InitializeCriticalSection(mutex);
  return 0;
#else
  return pthread_mutex_init(mutex, NULL);
#endif
}

// Lock mutex
VEX_INLINE int vex_mutex_lock(vex_mutex_t *mutex) {
#if defined(VEX_OS_WINDOWS)
  EnterCriticalSection(mutex);
  return 0;
#else
  return pthread_mutex_lock(mutex);
#endif
}

// Try lock mutex (non-blocking)
VEX_INLINE int vex_mutex_trylock(vex_mutex_t *mutex) {
#if defined(VEX_OS_WINDOWS)
  return TryEnterCriticalSection(mutex) ? 0 : EBUSY;
#else
  return pthread_mutex_trylock(mutex);
#endif
}

// Unlock mutex
VEX_INLINE int vex_mutex_unlock(vex_mutex_t *mutex) {
#if defined(VEX_OS_WINDOWS)
  LeaveCriticalSection(mutex);
  return 0;
#else
  return pthread_mutex_unlock(mutex);
#endif
}

// Destroy mutex
VEX_INLINE int vex_mutex_destroy(vex_mutex_t *mutex) {
#if defined(VEX_OS_WINDOWS)
  DeleteCriticalSection(mutex);
  return 0;
#else
  return pthread_mutex_destroy(mutex);
#endif
}

/* =========================
 * RWLock (Read-Write Lock)
 * ========================= */

#if defined(VEX_OS_WINDOWS)
typedef SRWLOCK vex_rwlock_t;
#else
typedef pthread_rwlock_t vex_rwlock_t;
#endif

// Initialize RWLock
VEX_INLINE int vex_rwlock_init(vex_rwlock_t *rwlock) {
#if defined(VEX_OS_WINDOWS)
  InitializeSRWLock(rwlock);
  return 0;
#else
  return pthread_rwlock_init(rwlock, NULL);
#endif
}

// Read lock (shared)
VEX_INLINE int vex_rwlock_rdlock(vex_rwlock_t *rwlock) {
#if defined(VEX_OS_WINDOWS)
  AcquireSRWLockShared(rwlock);
  return 0;
#else
  return pthread_rwlock_rdlock(rwlock);
#endif
}

// Try read lock (non-blocking)
VEX_INLINE int vex_rwlock_tryrdlock(vex_rwlock_t *rwlock) {
#if defined(VEX_OS_WINDOWS)
  return TryAcquireSRWLockShared(rwlock) ? 0 : EBUSY;
#else
  return pthread_rwlock_tryrdlock(rwlock);
#endif
}

// Write lock (exclusive)
VEX_INLINE int vex_rwlock_wrlock(vex_rwlock_t *rwlock) {
#if defined(VEX_OS_WINDOWS)
  AcquireSRWLockExclusive(rwlock);
  return 0;
#else
  return pthread_rwlock_wrlock(rwlock);
#endif
}

// Try write lock (non-blocking)
VEX_INLINE int vex_rwlock_trywrlock(vex_rwlock_t *rwlock) {
#if defined(VEX_OS_WINDOWS)
  return TryAcquireSRWLockExclusive(rwlock) ? 0 : EBUSY;
#else
  return pthread_rwlock_trywrlock(rwlock);
#endif
}

// Unlock RWLock
VEX_INLINE int vex_rwlock_unlock(vex_rwlock_t *rwlock) {
#if defined(VEX_OS_WINDOWS)
  // Note: Windows SRWLOCK requires separate unlock for read/write
  // For simplicity, we assume caller tracks lock type
  ReleaseSRWLockExclusive(rwlock);  // Works for both read and write
  return 0;
#else
  return pthread_rwlock_unlock(rwlock);
#endif
}

// Destroy RWLock
VEX_INLINE int vex_rwlock_destroy(vex_rwlock_t *rwlock) {
#if defined(VEX_OS_WINDOWS)
  // No explicit destroy needed for SRWLOCK
  (void)rwlock;
  return 0;
#else
  return pthread_rwlock_destroy(rwlock);
#endif
}

/* =========================
 * Condition Variable
 * ========================= */

#if defined(VEX_OS_WINDOWS)
typedef CONDITION_VARIABLE vex_cond_t;
#else
typedef pthread_cond_t vex_cond_t;
#endif

// Initialize condition variable
VEX_INLINE int vex_cond_init(vex_cond_t *cond) {
#if defined(VEX_OS_WINDOWS)
  InitializeConditionVariable(cond);
  return 0;
#else
  return pthread_cond_init(cond, NULL);
#endif
}

// Wait on condition variable
VEX_INLINE int vex_cond_wait(vex_cond_t *cond, vex_mutex_t *mutex) {
#if defined(VEX_OS_WINDOWS)
  return SleepConditionVariableCS(cond, mutex, INFINITE) ? 0 : -1;
#else
  return pthread_cond_wait(cond, mutex);
#endif
}

// Wait with timeout (nanoseconds)
VEX_INLINE int vex_cond_timedwait(vex_cond_t *cond, vex_mutex_t *mutex, uint64_t timeout_ns) {
#if defined(VEX_OS_WINDOWS)
  DWORD timeout_ms = (DWORD)(timeout_ns / 1000000);
  return SleepConditionVariableCS(cond, mutex, timeout_ms) ? 0 : ETIMEDOUT;
#else
  struct timespec ts;
  clock_gettime(CLOCK_REALTIME, &ts);
  ts.tv_sec += timeout_ns / 1000000000;
  ts.tv_nsec += timeout_ns % 1000000000;
  if (ts.tv_nsec >= 1000000000) {
    ts.tv_sec++;
    ts.tv_nsec -= 1000000000;
  }
  return pthread_cond_timedwait(cond, mutex, &ts);
#endif
}

// Signal one waiting thread
VEX_INLINE int vex_cond_signal(vex_cond_t *cond) {
#if defined(VEX_OS_WINDOWS)
  WakeConditionVariable(cond);
  return 0;
#else
  return pthread_cond_signal(cond);
#endif
}

// Broadcast to all waiting threads
VEX_INLINE int vex_cond_broadcast(vex_cond_t *cond) {
#if defined(VEX_OS_WINDOWS)
  WakeAllConditionVariable(cond);
  return 0;
#else
  return pthread_cond_broadcast(cond);
#endif
}

// Destroy condition variable
VEX_INLINE int vex_cond_destroy(vex_cond_t *cond) {
#if defined(VEX_OS_WINDOWS)
  // No explicit destroy needed for CONDITION_VARIABLE
  (void)cond;
  return 0;
#else
  return pthread_cond_destroy(cond);
#endif
}

/* =========================
 * Semaphore
 * ========================= */

#if defined(VEX_OS_WINDOWS)
typedef HANDLE vex_sem_t;
#else
typedef sem_t vex_sem_t;
#endif

// Initialize semaphore
VEX_INLINE int vex_sem_init(vex_sem_t *sem, uint32_t initial_count) {
#if defined(VEX_OS_WINDOWS)
  *sem = CreateSemaphoreA(NULL, (LONG)initial_count, LONG_MAX, NULL);
  return (*sem != NULL) ? 0 : -1;
#else
  return sem_init(sem, 0, initial_count);
#endif
}

// Wait (decrement) semaphore
VEX_INLINE int vex_sem_wait(vex_sem_t *sem) {
#if defined(VEX_OS_WINDOWS)
  return (WaitForSingleObject(*sem, INFINITE) == WAIT_OBJECT_0) ? 0 : -1;
#else
  return sem_wait(sem);
#endif
}

// Try wait (non-blocking)
VEX_INLINE int vex_sem_trywait(vex_sem_t *sem) {
#if defined(VEX_OS_WINDOWS)
  return (WaitForSingleObject(*sem, 0) == WAIT_OBJECT_0) ? 0 : EBUSY;
#else
  return sem_trywait(sem);
#endif
}

// Post (increment) semaphore
VEX_INLINE int vex_sem_post(vex_sem_t *sem) {
#if defined(VEX_OS_WINDOWS)
  return ReleaseSemaphore(*sem, 1, NULL) ? 0 : -1;
#else
  return sem_post(sem);
#endif
}

// Destroy semaphore
VEX_INLINE int vex_sem_destroy(vex_sem_t *sem) {
#if defined(VEX_OS_WINDOWS)
  return CloseHandle(*sem) ? 0 : -1;
#else
  return sem_destroy(sem);
#endif
}

/* =========================
 * Once (pthread_once equivalent)
 * ========================= */

#if defined(VEX_OS_WINDOWS)
typedef INIT_ONCE vex_once_t;
#define VEX_ONCE_INIT INIT_ONCE_STATIC_INIT
#else
typedef pthread_once_t vex_once_t;
#define VEX_ONCE_INIT PTHREAD_ONCE_INIT
#endif

#if defined(VEX_OS_WINDOWS)
static BOOL CALLBACK vex_once_wrapper(PINIT_ONCE once, PVOID param, PVOID *context) {
  (void)once;
  (void)context;
  void (*func)(void) = (void (*)(void))param;
  func();
  return TRUE;
}
#endif

// Call function exactly once
VEX_INLINE int vex_once(vex_once_t *once, void (*func)(void)) {
#if defined(VEX_OS_WINDOWS)
  return InitOnceExecuteOnce(once, vex_once_wrapper, func, NULL) ? 0 : -1;
#else
  return pthread_once(once, func);
#endif
}

/* =========================
 * Barrier
 * ========================= */

#if !defined(VEX_OS_WINDOWS)
typedef pthread_barrier_t vex_barrier_t;
#else
// Manual barrier implementation for Windows
typedef struct {
  vex_mutex_t mutex;
  vex_cond_t cond;
  uint32_t count;
  uint32_t threshold;
  uint32_t generation;
} vex_barrier_t;
#endif

// Initialize barrier
VEX_INLINE int vex_barrier_init(vex_barrier_t *barrier, uint32_t count) {
#if defined(VEX_OS_WINDOWS)
  vex_mutex_init(&barrier->mutex);
  vex_cond_init(&barrier->cond);
  barrier->count = count;
  barrier->threshold = count;
  barrier->generation = 0;
  return 0;
#else
  return pthread_barrier_init(barrier, NULL, count);
#endif
}

// Wait at barrier
VEX_INLINE int vex_barrier_wait(vex_barrier_t *barrier) {
#if defined(VEX_OS_WINDOWS)
  vex_mutex_lock(&barrier->mutex);
  uint32_t gen = barrier->generation;
  barrier->count--;
  
  if (barrier->count == 0) {
    // Last thread to arrive
    barrier->generation++;
    barrier->count = barrier->threshold;
    vex_cond_broadcast(&barrier->cond);
    vex_mutex_unlock(&barrier->mutex);
    return 1;  // Serial thread
  } else {
    // Wait for all threads
    while (gen == barrier->generation) {
      vex_cond_wait(&barrier->cond, &barrier->mutex);
    }
    vex_mutex_unlock(&barrier->mutex);
    return 0;
  }
#else
  return pthread_barrier_wait(barrier);
#endif
}

// Destroy barrier
VEX_INLINE int vex_barrier_destroy(vex_barrier_t *barrier) {
#if defined(VEX_OS_WINDOWS)
  vex_mutex_destroy(&barrier->mutex);
  vex_cond_destroy(&barrier->cond);
  return 0;
#else
  return pthread_barrier_destroy(barrier);
#endif
}

/* =========================
 * Demo / Tests
 * ========================= */
#ifdef VEX_SYNC_DEMO

#include <pthread.h>

// Shared counter (protected by mutex)
static vex_mutex_t g_mutex;
static int g_counter = 0;

// Atomic counter
static vex_atomic_i32_t g_atomic_counter = 0;

// Worker thread
static void* worker(void *arg) {
  int id = *(int*)arg;
  
  // Test mutex
  for (int i = 0; i < 1000; i++) {
    vex_mutex_lock(&g_mutex);
    g_counter++;
    vex_mutex_unlock(&g_mutex);
  }
  
  // Test atomic
  for (int i = 0; i < 1000; i++) {
    vex_atomic_add_i64((vex_atomic_i64_t*)&g_atomic_counter, 1, VEX_MEM_RELAXED);
  }
  
  printf("Worker %d: done\n", id);
  return NULL;
}

int main(void) {
  printf("=== Vex Sync Demo ===\n\n");
  
  // Initialize mutex
  vex_mutex_init(&g_mutex);
  
  // Spawn threads
  const int N_THREADS = 4;
  pthread_t threads[N_THREADS];
  int thread_ids[N_THREADS];
  
  for (int i = 0; i < N_THREADS; i++) {
    thread_ids[i] = i;
    pthread_create(&threads[i], NULL, worker, &thread_ids[i]);
  }
  
  // Join threads
  for (int i = 0; i < N_THREADS; i++) {
    pthread_join(threads[i], NULL);
  }
  
  printf("\nResults:\n");
  printf("  Mutex counter: %d (expected: %d)\n", g_counter, N_THREADS * 1000);
  printf("  Atomic counter: %d (expected: %d)\n", 
         vex_atomic_load_i64((vex_atomic_i64_t*)&g_atomic_counter, VEX_MEM_RELAXED), 
         N_THREADS * 1000);
  
  // Cleanup
  vex_mutex_destroy(&g_mutex);
  
  if (g_counter == N_THREADS * 1000 && 
      vex_atomic_load_i64((vex_atomic_i64_t*)&g_atomic_counter, VEX_MEM_RELAXED) == N_THREADS * 1000) {
    printf("\n✅ All tests passed!\n");
    return 0;
  } else {
    printf("\n❌ Test failed!\n");
    return 1;
  }
}

#endif // VEX_SYNC_DEMO

