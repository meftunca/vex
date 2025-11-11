# Concurrency

**Version:** 0.1.2 
**Last Updated:** November 2025

This document defines concurrency features in the Vex programming language.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1

---

## Concurrency Model

### Design Philosophy

Vex provides **multiple concurrency models**:

1. **Go-style goroutines**: Lightweight threads with CSP (Communicating Sequential Processes)
2. **Rust-style async/await**: Future-based asynchronous programming
3. **GPU computing**: Parallel execution on graphics processors

### Safety Guarantees

- **No data races**: Enforced by borrow checker
- **Thread safety**: Send/Sync traits (future)
- **Deadlock prevention**: Through ownership system

---

## Goroutines

### Syntax

**Keyword**: `go`

``````vex
go function_call();
```

**Properties**:

- Spawns lightweight concurrent task
- Similar to Go's goroutines
- Multiplexed onto OS threads
- Cheap to create (thousands possible)

### Examples (Conceptual)

**Simple Goroutine**:

[10 lines code: ```vex]

**With Arguments**:

[10 lines code: ```vex]

**Closure-like** (Future):

``````vex
fn main(): i32 {
    let data = 42;
    go  {
        // Use data
    };
    return 0;
}
```

### Current Status

**Parser**: ✅ Parses `go` statements 
**AST**: ✅ `Statement::Go(Expression)` node 
**Runtime**: ✅ Basic goroutine runtime implemented 
**Channels**: ✅ MPSC channels fully working

**Status**: Goroutines parsed and basic runtime operational. Work-stealing scheduler pending.

---

## Async/Await

### Syntax

**Keywords**: `async`, `await`

[9 lines code: ```vex]

### Async Functions

Define asynchronous functions:

``````vex
async fn download(url: string): (string | error) {
    // Non-blocking I/O
    let response = await http_get(url);
    return response;
}
```

**Properties**:

- Returns immediately (non-blocking)
- Returns a Future/Promise
- Must be awaited to get result
- Can be composed with other async functions

### Await Expressions

Wait for async result:

``````vex
async fn process(): i32 {
    let result1 = await operation1();
    let result2 = await operation2();
    return result1 + result2;
}
```

### Current Status

**Parser**: ✅ Parses `async fn` and `await` 
**AST**: ✅ `async` flag in Function, `await` expression 
**Runtime**: ✅ Basic async runtime implemented 
**Futures**: ✅ Basic Future support working

**Status**: Async/await syntax working with basic runtime. Advanced features (tokio integration, async I/O) pending.

---

## Channels

### Concept (Fully Implemented ✅)

Channels provide communication between concurrent tasks using CSP-style message passing:

[13 lines code: ```vex]

### Channel Operations

**Creation**:

``````vex
let! ch = Channel.new<i32>();        // Unbuffered channel
let! ch = Channel.new<i32>(100);     // Buffered channel (capacity 100)
```

**Sending**:

``````vex
ch.send(42);        // Send value (blocks if buffer full)
ch.try_send(42);    // Non-blocking send (returns bool)
```

**Receiving**:

``````vex
let value = ch.recv();              // Blocking receive (returns Option<T>)
let value = ch.try_recv();          // Non-blocking receive (returns Option<T>)
```

**Other Operations**:

``````vex
ch.close();        // Close channel
ch.is_closed();    // Check if closed
ch.len();          // Current buffer length
ch.capacity();     // Buffer capacity
```

### Current Status

**Syntax**: ✅ Fully defined 
**Implementation**: ✅ Complete (MPSC lock-free ring buffer) 
**Runtime**: ✅ C runtime (`vex_channel.c`) 
**Test Coverage**: ✅ 2 tests passing (`channel_simple.vx`, `channel_sync_test.vx`)

---

## GPU Computing

### Syntax (Parsed, Limited Support)

**Keyword**: `gpu`

``````vex
gpu fn matrix_multiply(a: [f32], b: [f32]): [f32] {
    // GPU kernel code
    // Executed in parallel on GPU
}
```

### GPU Functions

Define GPU kernels:

``````vex
gpu fn vector_add(a: [f32; 1024], b: [f32; 1024]): [f32; 1024] {
    // Parallel computation
    // Each thread processes one element
}
```

### Execution Model

**SIMT (Single Instruction, Multiple Threads)**:

- Same code runs on many threads
- Each thread has unique ID
- Threads grouped into blocks
- Blocks grouped into grid

### Current Status

**Parser**: ✅ Parses `gpu fn` declarations 
**AST**: ✅ `gpu` flag in Function 
**Backend**: ❌ No CUDA/OpenCL codegen 
**Runtime**: ❌ No GPU runtime

**Blocking Issues**:

1. Need CUDA/OpenCL backend
2. Need memory transfer primitives (host ↔ device)
3. Need thread indexing (`threadIdx`, `blockIdx`)
4. Need GPU runtime initialization

---

## Synchronization

### Mutex (Future)

Mutual exclusion lock:

``````vex
let mutex = Mutex::new(0);

fn increment() {
    let! guard = mutex.lock();
    *guard = *guard + 1;
    // Automatically unlocked when guard dropped
}
```

### RwLock (Future)

Read-write lock:

[11 lines code: ```vex]

### Atomic Operations (Future)

Lock-free synchronization:

``````vex
let counter = Atomic::new(0);

fn increment() {
    counter.fetch_add(1);  // Atomic increment
}
```

### WaitGroup (Future - Go-style)

Wait for goroutines to complete:

[11 lines code: ```vex]

### Current Status

**Mutex**: 🚧 Planned (Layer 2 std lib) 
**RwLock**: 🚧 Planned (Layer 2 std lib) 
**Atomic**: 🚧 Planned (Layer 2 std lib) 
**WaitGroup**: 🚧 Planned (Layer 2 std lib)

**Planned**: Layer 2 of standard library (sync module) - infrastructure ready, implementation pending

---

## Concurrency Patterns

### Fan-Out, Fan-In (Future)

Distribute work to multiple workers:

[15 lines code: ```vex]

### Pipeline (Future)

Chain processing stages:

[26 lines code: ```vex]

### Worker Pool (Future)

Fixed number of workers:

[29 lines code: ```vex]

---

## Thread Safety

### Send Trait (Future)

Types safe to transfer across threads:

[10 lines code: ```vex]

### Sync Trait (Future)

Types safe to share across threads:

``````vex
trait Sync { }

// Automatically implemented for immutable types
impl Sync for i32 { }

// Mutex makes T Sync
impl<T> Sync for Mutex<T> where T: Send { }
```

### Compiler Checks (Future)

[11 lines code: ```vex]

---

## Examples

### Goroutine (Conceptual)

[10 lines code: ```vex]

### Async/Await (Conceptual)

[9 lines code: ```vex]

### GPU Kernel (Conceptual)

[12 lines code: ```vex]

---

## Best Practices

### 1. Prefer Message Passing

[12 lines code: ```vex]

### 2. Keep Goroutines Simple

[9 lines code: ```vex]

### 3. Use Async for I/O

[13 lines code: ```vex]

### 4. Avoid Unnecessary Concurrency

[16 lines code: ```vex]

---

## Concurrency Summary

• Feature — Syntax — Status — Notes
• ------------------- — --------------- — -------------------- — --------------------------
| **Goroutines** | `go func()` | ✅ Basic runtime | Scheduler pending |
| **Async Functions** | `async fn` | ✅ Basic runtime | Advanced features pending |
| **Await** | `await expr` | ✅ Working | Basic support |
| **GPU Functions** | `gpu fn` | 🚧 Parsed | No backend |
| **Channels** | `channel<T>()` | ✅ Fully implemented | MPSC lock-free ring buffer |
| **Select** | `select { }` | 🚧 Keyword reserved | Syntax planned |
| **Mutex** | `Mutex::new()` | 🚧 Planned | Layer 2 std lib |
| **RwLock** | `RwLock::new()` | 🚧 Planned | Layer 2 std lib |
| **Atomic** | `Atomic::new()` | 🚧 Planned | Layer 2 std lib |
• **Send Trait** — Auto-derived — 🚧 Planned — Thread safety
• **Sync Trait** — Auto-derived — 🚧 Planned — Thread safety

### Implementation Status

**Syntax Level**: 60% complete (go, async, await, gpu parsed; channels working) 
**Runtime Level**: 40% complete (basic goroutines, async runtime, MPSC channels) 
**Library Level**: 0% complete (no sync primitives yet)

### Roadmap

**Phase 1: Async Runtime** (High Priority 🔴)

- Integrate tokio or custom runtime
- Implement Future trait
- Async I/O primitives
- Basic executor

**Phase 2: Goroutines** (High Priority 🔴)

- Work-stealing scheduler
- M:N threading model
- Stack management
- Runtime integration

**Phase 3: Channels** (Medium Priority 🟡)

- Channel implementation
- Select statement
- Buffered channels
- Broadcast channels

**Phase 4: GPU Computing** (Medium Priority 🟡)

- CUDA backend
- OpenCL backend
- Memory management (host ↔ device)
- Kernel launch primitives

**Phase 5: Sync Primitives** (Low Priority 🟢)

- Mutex, RwLock
- Atomic operations
- Semaphore, Barrier
- WaitGroup

---

**Previous**: \1 
**Next**: \1

**Maintained by**: Vex Language Team
