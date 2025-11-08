# Concurrency

**Version:** 0.9.2  
**Last Updated:** November 2025

This document defines concurrency features in the Vex programming language.

---

## Table of Contents

1. [Concurrency Model](#concurrency-model)
2. [Goroutines](#goroutines)
3. [Async/Await](#asyncawait)
4. [Channels](#channels)
5. [GPU Computing](#gpu-computing)
6. [Synchronization](#synchronization)

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

```vex
go function_call();
```

**Properties**:

- Spawns lightweight concurrent task
- Similar to Go's goroutines
- Multiplexed onto OS threads
- Cheap to create (thousands possible)

### Examples (Conceptual)

**Simple Goroutine**:

```vex
fn worker() {
    // Do work concurrently
}

fn main(): i32 {
    go worker();  // Spawn goroutine
    go worker();  // Spawn another
    // Continue main thread
    return 0;
}
```

**With Arguments**:

```vex
fn process(id: i32) {
    // Process with id
}

fn main(): i32 {
    go process(1);
    go process(2);
    go process(3);
    return 0;
}
```

**Closure-like** (Future):

```vex
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
**Runtime**: ❌ No scheduler implementation  
**Channels**: ❌ Not implemented

**Blocking Issues**:

1. Need goroutine scheduler (work-stealing queue)
2. Need runtime integration (similar to Go's runtime)
3. Need channel implementation for communication
4. Need `sync` package primitives

---

## Async/Await

### Syntax

**Keywords**: `async`, `await`

```vex
async fn fetch_data(url: string): string {
    // Asynchronous operation
    return "data";
}

async fn main(): i32 {
    let data = await fetch_data("https://api.example.com");
    return 0;
}
```

### Async Functions

Define asynchronous functions:

```vex
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

```vex
async fn process(): i32 {
    let result1 = await operation1();
    let result2 = await operation2();
    return result1 + result2;
}
```

### Current Status

**Parser**: ✅ Parses `async fn` and `await`  
**AST**: ✅ `async` flag in Function, `await` expression  
**Runtime**: ❌ No async runtime (tokio integration pending)  
**Futures**: ❌ Not implemented

**Blocking Issues**:

1. Need async runtime (likely tokio-based)
2. Need Future trait implementation
3. Need async I/O primitives
4. Need executor and reactor

---

## Channels

### Concept (Fully Implemented ✅)

Channels provide communication between concurrent tasks using CSP-style message passing:

```vex
// Create channel
let! ch = Channel.new<i32>();

// Send value
go {
    ch.send(42);
};

// Receive value
match ch.recv() {
    Option.Some(value) => println("Received: {}", value),
    Option.None => println("Channel empty"),
}
```

### Channel Operations

**Creation**:

```vex
let! ch = Channel.new<i32>();        // Unbuffered channel
let! ch = Channel.new<i32>(100);     // Buffered channel (capacity 100)
```

**Sending**:

```vex
ch.send(42);        // Send value (blocks if buffer full)
ch.try_send(42);    // Non-blocking send (returns bool)
```

**Receiving**:

```vex
let value = ch.recv();              // Blocking receive (returns Option<T>)
let value = ch.try_recv();          // Non-blocking receive (returns Option<T>)
```

**Other Operations**:

```vex
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

```vex
gpu fn matrix_multiply(a: [f32], b: [f32]): [f32] {
    // GPU kernel code
    // Executed in parallel on GPU
}
```

### GPU Functions

Define GPU kernels:

```vex
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

```vex
let mutex = Mutex::new(0);

fn increment() {
    let! guard = mutex.lock();
    *guard = *guard + 1;
    // Automatically unlocked when guard dropped
}
```

### RwLock (Future)

Read-write lock:

```vex
let rwlock = RwLock::new(vec![]);

fn read_data(): [i32] {
    let guard = rwlock.read();
    return *guard;
}

fn write_data(data: [i32]) {
    let! guard = rwlock.write();
    *guard = data;
}
```

### Atomic Operations (Future)

Lock-free synchronization:

```vex
let counter = Atomic::new(0);

fn increment() {
    counter.fetch_add(1);  // Atomic increment
}
```

### WaitGroup (Future - Go-style)

Wait for goroutines to complete:

```vex
let wg = WaitGroup::new();

for i in 0..10 {
    wg.add(1);
    go  {
        defer wg.done();
        // Do work
    };
}

wg.wait();  // Wait for all goroutines
```

### Current Status

**Mutex**: ❌ Not implemented  
**RwLock**: ❌ Not implemented  
**Atomic**: ❌ Not implemented  
**WaitGroup**: ❌ Not implemented

**Planned**: Layer 2 of standard library (sync module)

---

## Concurrency Patterns

### Fan-Out, Fan-In (Future)

Distribute work to multiple workers:

```vex
fn fan_out(work: [Task]) {
    let (tx, rx) = channel<Result>();

    for task in work {
        go  {
            let result = process(task);
            tx.send(result);
        };
    }

    // Collect results
    for i in 0..work.len() {
        let result = rx.recv();
    }
}
```

### Pipeline (Future)

Chain processing stages:

```vex
fn pipeline(input: [Data]): [Result] {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    // Stage 1
    go  {
        for data in input {
            tx1.send(process_stage1(data));
        }
    };

    // Stage 2
    go  {
        loop {
            let data = rx1.recv();
            tx2.send(process_stage2(data));
        }
    };

    // Collect
    let! results = [];
    for i in 0..input.len() {
        results.push(rx2.recv());
    }
    return results;
}
```

### Worker Pool (Future)

Fixed number of workers:

```vex
fn worker_pool(work: [Task], num_workers: i32): [Result] {
    let (work_tx, work_rx) = channel<Task>();
    let (result_tx, result_rx) = channel<Result>();

    // Spawn workers
    for i in 0..num_workers {
        go  {
            loop {
                let task = work_rx.recv();
                let result = process(task);
                result_tx.send(result);
            }
        };
    }

    // Send work
    go  {
        for task in work {
            work_tx.send(task);
        }
    };

    // Collect results
    let! results = [];
    for i in 0..work.len() {
        results.push(result_rx.recv());
    }
    return results;
}
```

---

## Thread Safety

### Send Trait (Future)

Types safe to transfer across threads:

```vex
trait Send { }

// Automatically implemented for types without references
impl Send for i32 { }
impl Send for Point { }

// Not Send: contains non-Send types
struct HasReference {
    data: &i32,  // Not Send
}
```

### Sync Trait (Future)

Types safe to share across threads:

```vex
trait Sync { }

// Automatically implemented for immutable types
impl Sync for i32 { }

// Mutex makes T Sync
impl<T> Sync for Mutex<T> where T: Send { }
```

### Compiler Checks (Future)

```vex
fn spawn<F: Send>(f: F) {
    // Compiler ensures F is Send
    go f();
}

let x = 42;
// ERROR: x is not Send (contains reference)
let ref_x = &x;
spawn(|| {
    // use ref_x  // Compile error!
});
```

---

## Examples

### Goroutine (Conceptual)

```vex
fn worker(id: i32) {
    // Do work
}

fn main(): i32 {
    go worker(1);
    go worker(2);
    go worker(3);
    return 0;
}
```

### Async/Await (Conceptual)

```vex
async fn fetch(url: string): string {
    return "data";
}

async fn main(): i32 {
    let data1 = await fetch("url1");
    let data2 = await fetch("url2");
    return 0;
}
```

### GPU Kernel (Conceptual)

```vex
gpu fn add_vectors(a: [f32; 1024], b: [f32; 1024]): [f32; 1024] {
    let! result: [f32; 1024];
    // Each GPU thread processes one element
    return result;
}

fn main(): i32 {
    let a: [f32; 1024];
    let b: [f32; 1024];
    let result = add_vectors(a, b);
    return 0;
}
```

---

## Best Practices

### 1. Prefer Message Passing

```vex
// Good: Use channels (future)
let (tx, rx) = channel<i32>();
go  {
    tx.send(42);
};
let value = rx.recv();

// Bad: Shared mutable state
let! shared = 0;
go  {
    shared = 42;  // Data race!
};
```

### 2. Keep Goroutines Simple

```vex
// Good: Simple, focused task
go process_item(item);

// Bad: Complex logic in goroutine
go  {
    // Hundreds of lines
    // Complex error handling
    // Multiple responsibilities
};
```

### 3. Use Async for I/O

```vex
// Good: Async for I/O-bound work
async fn fetch_all(urls: [string]) {
    for url in urls {
        await fetch(url);
    }
}

// Bad: Goroutines for I/O (less efficient)
fn fetch_all(urls: [string]) {
    for url in urls {
        go fetch_blocking(url);
    }
}
```

### 4. Avoid Unnecessary Concurrency

```vex
// Good: Sequential when appropriate
fn process_small_list(items: [i32; 10]): [i32; 10] {
    let! results: [i32; 10];
    for i in 0..10 {
        results[i] = process(items[i]);
    }
    return results;
}

// Bad: Overhead of concurrency
fn process_small_list(items: [i32; 10]): [i32; 10] {
    // Goroutine overhead larger than computation!
    for item in items {
        go process(item);
    }
}
```

---

## Concurrency Summary

| Feature             | Syntax          | Status               | Notes           |
| ------------------- | --------------- | -------------------- | --------------- |
| **Goroutines**      | `go func()`     | 🚧 Parsed            | No runtime      |
| **Async Functions** | `async fn`      | 🚧 Parsed            | No runtime      |
| **Await**           | `await expr`    | 🚧 Parsed            | No runtime      |
| **GPU Functions**   | `gpu fn`        | 🚧 Parsed            | No backend      |
| **Channels**        | `channel<T>()`  | ✅ Fully implemented |                 |
| **Select**          | `select { }`    | ❌ Not defined       | Planned         |
| **Mutex**           | `Mutex::new()`  | ❌ Not implemented   | Layer 2 std lib |
| **RwLock**          | `RwLock::new()` | ❌ Not implemented   | Layer 2 std lib |
| **Atomic**          | `Atomic::new()` | ❌ Not implemented   | Layer 2 std lib |
| **Send Trait**      | Auto-derived    | ❌ Not implemented   | Thread safety   |
| **Sync Trait**      | Auto-derived    | ❌ Not implemented   | Thread safety   |

### Implementation Status

**Syntax Level**: 30% complete (go, async, await, gpu parsed)  
**Runtime Level**: 0% complete (no scheduler, no executor)  
**Library Level**: 0% complete (no sync primitives)

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

**Previous**: [12_Memory_Management.md](./12_Memory_Management.md)  
**Next**: [14_Modules_and_Imports.md](./14_Modules_and_Imports.md)

**Maintained by**: Vex Language Team
