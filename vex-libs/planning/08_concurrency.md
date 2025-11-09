# Vex Stdlib Planning - 08: Concurrency

**Priority:** 8
**Status:** Partial (sync exists, context missing)
**Dependencies:** builtin, unsafe

## 📦 Packages in This Category

### 8.1 sync
**Status:** ✅ Exists (extend with missing primitives)
**Description:** Synchronization primitives

#### Current Implementation
- Basic sync primitives exist

#### Required Extensions
```vex
// Mutex
struct Mutex {
    // internal state
}

fn new_mutex(): Mutex
fn lock(m: &Mutex)
fn unlock(m: &Mutex)
fn try_lock(m: &Mutex): bool

// RWMutex
struct RWMutex {
    // internal state
}

fn new_rwmutex(): RWMutex
fn r_lock(rw: &RWMutex)
fn r_unlock(rw: &RWMutex)
fn lock(rw: &RWMutex)
fn unlock(rw: &RWMutex)
fn try_r_lock(rw: &RWMutex): bool
fn try_lock(rw: &RWMutex): bool

// WaitGroup
struct WaitGroup {
    // internal state
}

fn new_wait_group(): WaitGroup
fn add(wg: &WaitGroup, delta: i32)
fn done(wg: &WaitGroup)
fn wait(wg: &WaitGroup)

// Once
struct Once {
    // internal state
}

fn new_once(): Once
fn do(o: &Once, f: fn())

// Cond
struct Cond {
    l: Locker,
    // internal
}

fn new_cond(l: Locker): *Cond
fn wait(c: *Cond)
fn signal(c: *Cond)
fn broadcast(c: *Cond)

// Pool
struct Pool {
    new: fn(): any,
    // internal
}

fn new_pool(new: fn(): any): *Pool
fn get(p: *Pool): any
fn put(p: *Pool, x: any)

// Map
struct Map {
    // concurrent safe map
}

fn new_map(): Map
fn load(m: &Map, key: any): (any, bool)
fn store(m: &Map, key: any, value: any)
fn load_or_store(m: &Map, key: any, value: any): (any, bool)
fn delete(m: &Map, key: any)
fn range(m: &Map, f: fn(key: any, value: any): bool)
```

#### Required Interfaces
```vex
trait Locker {
    fn lock(self: &Self)
    fn unlock(self: &Self)
}
```

#### Dependencies
- builtin
- unsafe

### 8.2 context
**Status:** ❌ Missing (critical for request handling)
**Description:** Context propagation and cancellation

#### Required Types
```vex
trait Context {
    fn deadline(self: &Self): (time.Time, bool)
    fn done(self: &Self): <-chan struct{}
    fn err(self: &Self): Error
    fn value(self: &Self, key: any): any
}

struct CancelFunc {
    // function pointer
}

struct emptyCtx {
    // implements Context
}

struct cancelCtx {
    parent: Context,
    children: sync.Map,
    err: Error,
    done: chan struct{},
}

struct timerCtx {
    cancel_ctx: cancelCtx,
    timer: *time.Timer,
    deadline: time.Time,
}

struct valueCtx {
    parent: Context,
    key: any,
    val: any,
}
```

#### Required Functions
```vex
// Context constructors
fn background(): Context
fn todo(): Context
fn with_cancel(parent: Context): (Context, CancelFunc)
fn with_deadline(parent: Context, d: time.Time): (Context, CancelFunc)
fn with_timeout(parent: Context, timeout: time.Duration): (Context, CancelFunc)
fn with_value(parent: Context, key: any, val: any): Context

// Cancellation
fn cancel(cancel_func: CancelFunc)

// Context operations
fn cause(c: Context): Error
```

#### Dependencies
- builtin
- time
- sync

#### Notes
- **Channels:** Heavy use of channels for cancellation
- **Goroutines:** Context propagation across goroutines

### 8.3 atomic
**Status:** ❌ Missing (important for lock-free programming)
**Description:** Atomic operations

#### Required Functions
```vex
// Load operations
fn load_int32(addr: *i32): i32
fn load_int64(addr: *i64): i64
fn load_uint32(addr: *u32): u32
fn load_uint64(addr: *u64): u64
fn load_uintptr(addr: *usize): usize
fn load_pointer(addr: **u8): *u8

// Store operations
fn store_int32(addr: *i32, val: i32)
fn store_int64(addr: *i64, val: i64)
fn store_uint32(addr: *u32, val: i32)
fn store_uint64(addr: *u64, val: i64)
fn store_uintptr(addr: *usize, val: usize)
fn store_pointer(addr: **u8, val: *u8)

// Swap operations
fn swap_int32(addr: *i32, new: i32): i32
fn swap_int64(addr: *i64, new: i64): i64
fn swap_uint32(addr: *u32, new: u32): u32
fn swap_uint64(addr: *u64, new: u64): u64
fn swap_uintptr(addr: *usize, new: usize): usize
fn swap_pointer(addr: **u8, new: *u8): *u8

// Compare and swap
fn compare_and_swap_int32(addr: *i32, old: i32, new: i32): bool
fn compare_and_swap_int64(addr: *i64, old: i64, new: i64): bool
fn compare_and_swap_uint32(addr: *u32, old: u32, new: u32): bool
fn compare_and_swap_uint64(addr: *u64, old: u64, new: u64): bool
fn compare_and_swap_uintptr(addr: *usize, old: usize, new: usize): bool
fn compare_and_swap_pointer(addr: **u8, old: *u8, new: *u8): bool

// Add operations
fn add_int32(addr: *i32, delta: i32): i32
fn add_int64(addr: *i64, delta: i64): i64
fn add_uint32(addr: *u32, delta: u32): u32
fn add_uint64(addr: *u64, delta: u64): u64
fn add_uintptr(addr: *usize, delta: usize): usize
```

#### Dependencies
- builtin
- unsafe

### 8.4 Ownership Model Extensions (Rust-inspired)
**Status:** ❌ Missing (critical for memory safety)
**Description:** Rust-style ownership and borrowing for Vex

#### Required Concepts
```vex
// Ownership transfer
fn move<T>(value: T): T  // Explicit move
fn clone<T: Clone>(value: &T): T  // Explicit clone

// Borrowing
fn borrow<T>(value: &T): &T  // Immutable borrow
fn borrow_mut<T>(value: &mut T): &mut T  // Mutable borrow

// Lifetime management
struct Ref<'a, T> {
    data: *mut T,
    _marker: PhantomData<&'a T>,
}

struct RefMut<'a, T> {
    data: *mut T,
    _marker: PhantomData<&'a mut T>,
}

struct RefCell<T> {
    borrow: Cell<usize>,
    value: UnsafeCell<T>,
}

struct Rc<T> {
    ptr: *mut RcBox<T>,
}

struct Arc<T> {
    ptr: *mut ArcInner<T>,
}

// Smart pointers
impl<T> Deref for Box<T> {
    type Target = T;
    fn deref(&self): &T
}

impl<T> Deref for Rc<T> {
    type Target = T;
    fn deref(&self): &T
}

impl<T> Deref for Arc<T> {
    type Target = T;
    fn deref(&self): &T
}
```

#### Required Traits
```vex
trait Clone {
    fn clone(&self): Self;
}

trait Copy: Clone {}  // Marker trait for copy types

trait Drop {
    fn drop(&mut self);
}

trait Deref {
    type Target;
    fn deref(&self): &Self::Target;
}

trait DerefMut: Deref {
    fn deref_mut(&mut self): &mut Self::Target;
}

trait Borrow<Borrowed> {
    fn borrow(&self): &Borrowed;
}

trait BorrowMut<Borrowed>: Borrow<Borrowed> {
    fn borrow_mut(&mut self): &mut Borrowed;
}

trait ToOwned {
    type Owned;
    fn to_owned(&self): Self::Owned;
}
```

#### Required Functions
```vex
// Box operations
fn box_new<T>(value: T): Box<T>
fn box_into_inner<T>(b: Box<T>): T

// Rc operations
fn rc_new<T>(value: T): Rc<T>
fn rc_clone<T>(rc: &Rc<T>): Rc<T>
fn rc_strong_count<T>(rc: &Rc<T>): usize
fn rc_weak_count<T>(rc: &Rc<T>): usize

// Arc operations
fn arc_new<T>(value: T): Arc<T>
fn arc_clone<T>(arc: &Arc<T>): Arc<T>
fn arc_strong_count<T>(arc: &Arc<T>): usize
fn arc_weak_count<T>(arc: &Arc<T>): usize

// RefCell operations
fn refcell_new<T>(value: T): RefCell<T>
fn refcell_borrow<T>(rc: &RefCell<T>): Ref<T>
fn refcell_borrow_mut<T>(rc: &RefCell<T>): RefMut<T>
fn refcell_try_borrow<T>(rc: &RefCell<T>): Result<Ref<T>, BorrowError>
fn refcell_try_borrow_mut<T>(rc: &RefCell<T>): Result<RefMut<T>, BorrowMutError>
```

#### Dependencies
- builtin
- marker
- ops
- cell

#### Notes
- **Borrow Checker Integration:** Vex'in mevcut borrow checker'ını genişlet
- **Lifetime Parameters:** Generic lifetime support gerekli
- **Move Semantics:** Ownership transfer semantics

## 🎯 Implementation Priority

1. **sync extensions** - Complete synchronization primitives
2. **context** - Context propagation and cancellation
3. **atomic** - Lock-free atomic operations
4. **ownership model** - Rust-style ownership and borrowing

## ⚠️ Language Feature Issues

- **Channels:** Extensive use of channels in context
- **Goroutines:** Required for concurrent operations
- **Atomic Builtins:** May need compiler intrinsics
- **Lifetime System:** Advanced lifetime management
- **Move Semantics:** Ownership transfer rules

## 📋 Missing Critical Dependencies

- **Goroutine Runtime:** For concurrent execution
- **Channel Types:** For communication
- **Atomic Intrinsics:** Hardware atomic operations
- **Lifetime Checker:** Advanced borrow checking
- **Move Analyzer:** Ownership transfer analysis

## 🚀 Next Steps

1. Extend sync package with missing primitives
2. Implement context package
3. Add atomic operations
4. Integrate Rust ownership model
5. Enhance borrow checker with lifetimes