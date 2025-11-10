# Vex Stdlib Planning - Rust Stdlib Additions

**Version:** 0.1.2
**Date:** November 9, 2025
**Inspired by:** Rust Standard Library
**Integration:** Extends Go-based planning with Rust-specific features

## 🎯 Rust Stdlib Integration Goals

Rust'ın std kütüphanesinden Vex'e uygun olan özellikleri entegre et:

- **Memory Safety:** Ownership ve borrowing modelini güçlendir
- **Zero-Cost Abstractions:** Performans odaklı tasarım
- **Traits System:** Generic programming için trait'ler
- **Error Handling:** Result/Option pattern'ini genişlet
- **Collections:** Rust'ın gelişmiş koleksiyonları

## 📦 Rust-Specific Packages to Add

### Core Traits and Types (01_core_builtin.md'ya eklenecek)

#### ops (Operator Overloading)

**Status:** ❌ Missing (critical for Rust-like ergonomics)
**Description:** Operator overloading traits

```vex
trait Add<Rhs = Self> {
    type Output;
    fn add(self: Self, rhs: Rhs): Self::Output;
}

trait Sub<Rhs = Self> {
    type Output;
    fn sub(self: Self, rhs: Rhs): Self::Output;
}

trait Mul<Rhs = Self> {
    type Output;
    fn mul(self: Self, rhs: Rhs): Self::Output;
}

trait Div<Rhs = Self> {
    type Output;
    fn div(self: Self, rhs: Rhs): Self::Output;
}

trait Rem<Rhs = Self> {
    type Output;
    fn rem(self: Self, rhs: Rhs): Self::Output;
}

trait Neg {
    type Output;
    fn neg(self: Self): Self::Output;
}

trait Not {
    type Output;
    fn not(self: Self): Self::Output;
}

// Comparison operators
trait PartialEq<Rhs = Self> {
    fn eq(self: &Self, other: &Rhs): bool;
    fn ne(self: &Self, other: &Rhs): bool { !self.eq(other) }
}

trait Eq: PartialEq<Self> {}

trait PartialOrd<Rhs = Self>: PartialEq<Rhs> {
    fn partial_cmp(self: &Self, other: &Rhs): Option<Ordering>;
    fn lt(self: &Self, other: &Rhs): bool { matches!(self.partial_cmp(other), Some(Less)) }
    fn le(self: &Self, other: &Rhs): bool { matches!(self.partial_cmp(other), Some(Less | Equal)) }
    fn gt(self: &Self, other: &Rhs): bool { matches!(self.partial_cmp(other), Some(Greater)) }
    fn ge(self: &Self, other: &Rhs): bool { matches!(self.partial_cmp(other), Some(Greater | Equal)) }
}

trait Ord: Eq + PartialOrd<Self> {
    fn cmp(self: &Self, other: &Self): Ordering;
}

enum Ordering {
    Less,
    Equal,
    Greater,
}
```

#### convert (Type Conversion)

**Status:** ❌ Missing (important for generic programming)
**Description:** Type conversion traits

```vex
trait From<T> {
    fn from(value: T): Self;
}

trait Into<T> {
    fn into(self: Self): T;
}

trait TryFrom<T> {
    type Error;
    fn try_from(value: T): Result<Self, Self::Error>;
}

trait TryInto<T> {
    type Error;
    fn try_into(self: Self): Result<T, Self::Error>;
}

trait AsRef<T> {
    fn as_ref(self: &Self): &T;
}

trait AsMut<T> {
    fn as_mut(self: &mut Self): &mut T;
}
```

#### marker (Marker Traits)

**Status:** ❌ Missing (critical for concurrency)
**Description:** Special marker traits for type system

```vex
trait Sized {}  // Most types are sized

trait Copy: Clone {}  // Types that can be copied by simple memcpy

trait Send {}  // Types that can be transferred across thread boundaries

trait Sync {}  // Types for which &T is Send

trait Unpin {}  // Types that don't care about pin

trait Drop {
    fn drop(self: &mut Self);
}
```

#### default (Default Values)

**Status:** ❌ Missing (useful for initialization)
**Description:** Default value generation

```vex
trait Default {
    fn default(): Self;
}

// Auto-implement for basic types
impl Default for i32 { fn default(): i32 { 0 } }
impl Default for bool { fn default(): bool { false } }
impl Default for str { fn default(): str { "" } }
```

#### hash (Hashing)

**Status:** Partial (SipHasher missing, basic hash exists)
**C Code Status:** ❌ vex_hash.c missing (SipHasher implementation)
**Description:** Hashing traits and utilities

```vex
trait Hash {
    fn hash<H: Hasher>(self: &Self, state: &mut H);
}

trait Hasher {
    fn write(self: &mut Self, bytes: &[u8]);
    fn finish(self: &Self): u64;
}

// Hash functions
struct SipHasher { /* internal */ }
struct DefaultHasher { /* internal */ }

fn hash<T: Hash>(value: &T): u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
```

### Memory Management (Yeni kategori - alloc)

#### alloc (Memory Allocation)

**Status:** Partial (Box exists, Rc/Arc missing)
**C Code Status:** ✅ vex_box.c exists, ❌ vex_smartptr.c missing (Rc/Arc)
**Description:** Heap allocation and collections

```vex
// Box (unique ownership)
struct Box<T> {
    ptr: *mut T,
}

fn box_new<T>(value: T): Box<T>
fn box_into_inner<T>(b: Box<T>): T

// Rc (reference counted)
struct Rc<T> {
    ptr: *mut RcBox<T>,
}

struct RcBox<T> {
    strong: usize,
    weak: usize,
    value: T,
}

fn rc_new<T>(value: T): Rc<T>
fn rc_clone<T>(rc: &Rc<T>): Rc<T>
fn rc_strong_count<T>(rc: &Rc<T>): usize

// Arc (atomic reference counted)
struct Arc<T> {
    ptr: *mut ArcInner<T>,
}

fn arc_new<T>(value: T): Arc<T>
fn arc_clone<T>(arc: &Arc<T>): Arc<T>
```

### Collections Extensions (03_collections_algorithms.md'ye eklenecek)

#### collections (Rust-style collections)

**Status:** Partial (HashMap exists via swisstable, others missing)
**C Code Status:** ✅ vex_swisstable.c (HashMap), ❌ vex_vecdeque.c, vex_linkedlist.c missing
**Description:** Rust's advanced collections

```vex
// HashMap (Rust's HashMap)
struct HashMap<K, V> {
    // internal hash table
}

fn hashmap_new<K, V>(): HashMap<K, V>
fn hashmap_insert<K: Hash + Eq, V>(map: &mut HashMap<K, V>, k: K, v: V): Option<V>
fn hashmap_get<K: Hash + Eq, V>(map: &HashMap<K, V>, k: &K): Option<&V>
fn hashmap_remove<K: Hash + Eq, V>(map: &mut HashMap<K, V>, k: &K): Option<V>

// HashSet
struct HashSet<T> {
    map: HashMap<T, ()>,
}

fn hashset_new<T>(): HashSet<T>
fn hashset_insert<T: Hash + Eq>(set: &mut HashSet<T>, value: T): bool
fn hashset_contains<T: Hash + Eq>(set: &HashSet<T>, value: &T): bool

// VecDeque (double-ended queue)
struct VecDeque<T> {
    buf: Vec<T>,
    head: usize,
    len: usize,
}

fn vecdeque_new<T>(): VecDeque<T>
fn vecdeque_push_front<T>(deque: &mut VecDeque<T>, value: T)
fn vecdeque_push_back<T>(deque: &mut VecDeque<T>, value: T)
fn vecdeque_pop_front<T>(deque: &mut VecDeque<T>): Option<T>
fn vecdeque_pop_back<T>(deque: &mut VecDeque<T>): Option<T>

// LinkedList
struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
    element: T,
}
```

### String and Text Extensions (04_strings_text.md'ye eklenecek)

#### regex (Regular Expressions)

**Status:** ✅ Exists
**C Code Status:** ✅ vex_regex.c exists
**Description:** Regular expression matching

```vex
struct Regex {
    // compiled regex
}

struct Captures<'t> {
    text: &'t str,
    names: Vec<Option<usize>>,
    matches: Vec<Option<(usize, usize)>>,
}

struct Match<'t> {
    text: &'t str,
    start: usize,
    end: usize,
}

fn regex_new(pattern: str): Result<Regex, Error>
fn is_match(regex: &Regex, text: str): bool
fn find(regex: &Regex, text: str): Option<Match>
fn find_iter(regex: &Regex, text: str): Matches
fn captures(regex: &Regex, text: str): Option<Captures>
fn captures_iter(regex: &Regex, text: str): CaptureMatches
fn replace(regex: &Regex, text: str, rep: str): str
fn replace_all(regex: &Regex, text: str, rep: str): str

// Regex builder
struct RegexBuilder {
    pattern: str,
    case_insensitive: bool,
    multi_line: bool,
    dot_matches_new_line: bool,
    swap_greed: bool,
    ignore_whitespace: bool,
    unicode: bool,
    octal: bool,
}

fn regex_builder(): RegexBuilder
fn build(builder: RegexBuilder): Result<Regex, Error>
```

### System and OS Extensions (06_system_os.md'ye eklenecek)

#### process (Process Management)

**Status:** ✅ Exists
**C Code Status:** ✅ vex_cmd.c exists
**Description:** Process spawning and management

```vex
struct Command {
    program: str,
    args: Vec<str>,
    env: Vec<str>,
    cwd: Option<PathBuf>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
}

enum Stdio {
    Inherit,
    Piped,
    Null,
}

struct Child {
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    // internal
}

struct Output {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

struct ExitStatus {
    code: Option<i32>,
}

fn command(program: str): Command
fn arg(cmd: &mut Command, arg: str): &mut Command
fn args(cmd: &mut Command, args: &[str]): &mut Command
fn env(cmd: &mut Command, key: str, val: str): &mut Command
fn spawn(cmd: &Command): Result<Child, Error>
fn output(cmd: &Command): Result<Output, Error>
fn status(cmd: &Command): Result<ExitStatus, Error>
```

#### thread (Thread Management)

**Status:** ✅ Exists (basic threading)
**C Code Status:** ✅ vex_testing.c, vex_cmd.c (threading functions)
**Description:** Thread creation and management

```vex
struct JoinHandle<T> {
    // internal
}

struct Thread {
    // internal
}

fn spawn<F, T>(f: F): JoinHandle<T> where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static
fn current(): Thread
fn park()
fn unpark(thread: &Thread)
fn yield_now()
fn sleep(dur: Duration)
fn available_parallelism(): usize
```

### Panic and Error Handling Extensions

#### panic (Panic Handling)

**Status:** ✅ Exists (basic panic)
**C Code Status:** ✅ vex_panic functions in vex_array.c, vex_result.c, etc.
**Description:** Panic handling and recovery

```vex
fn panic_any(payload: Box<dyn Any>) -> !
fn catch_unwind<F, R>(f: F): Result<R, Box<dyn Any>> where F: FnOnce() -> R
fn resume_unwind(payload: Box<dyn Any>) -> !
fn set_hook(hook: fn(&PanicInfo))
fn take_hook(): fn(&PanicInfo)

struct PanicInfo {
    payload: Box<dyn Any>,
    message: Option<&'static str>,
    location: Option<&Location>,
}

struct Location {
    file: &'static str,
    line: u32,
    col: u32,
}
```

#### backtrace (Stack Traces)

**Status:** ❌ Missing
**C Code Status:** ❌ vex_backtrace.c missing
**Description:** Stack trace capture and printing

```vex
struct Backtrace {
    // internal
}

fn backtrace() -> Backtrace
fn capture() -> Backtrace
fn status(bt: &Backtrace): BacktraceStatus

enum BacktraceStatus {
    Unsupported,
    Disabled,
    Captured,
}

fn frames(bt: &Backtrace) -> &[BacktraceFrame]
fn print(bt: &Backtrace, fmt: &mut fmt::Formatter) -> fmt::Result
```

### Iterator Extensions (Yeni kategori - iter)

#### iter (Iterators)

**Status:** ❌ Missing (Rust's powerful iterator system)
**Description:** Iterator traits and utilities

```vex
trait Iterator {
    type Item;
    fn next(self: &mut Self): Option<Self::Item>;
}

trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    fn into_iter(self: Self): Self::IntoIter;
}

trait FromIterator<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T): Self;
}

// Iterator adapters
fn map<B, F>(self: Self, f: F) -> Map<Self, F> where F: FnMut(Self::Item) -> B
fn filter<P>(self: Self, predicate: P) -> Filter<Self, P> where P: FnMut(&Self::Item) -> bool
fn filter_map<B, F>(self: Self, f: F) -> FilterMap<Self, F> where F: FnMut(Self::Item) -> Option<B>
fn chain<U>(self: Self, other: U) -> Chain<Self, U::IntoIter> where U: IntoIterator<Item = Self::Item>
fn zip<U>(self: Self, other: U) -> Zip<Self, U::IntoIter> where U: IntoIterator
fn take(self: Self, n: usize) -> Take<Self>
fn skip(self: Self, n: usize) -> Skip<Self>
fn enumerate(self: Self) -> Enumerate<Self>
fn collect<B>(self: Self) -> B where B: FromIterator<Self::Item>
fn count(self: Self) -> usize
fn fold<B, F>(self: Self, init: B, f: F) -> B where F: FnMut(B, Self::Item) -> B
fn all<F>(self: Self, f: F) -> bool where F: FnMut(Self::Item) -> bool
fn any<F>(self: Self, f: F) -> bool where F: FnMut(Self::Item) -> bool
fn find<P>(self: Self, predicate: P) -> Option<Self::Item> where P: FnMut(&Self::Item) -> bool
fn position<P>(self: Self, predicate: P) -> Option<usize> where P: FnMut(Self::Item) -> bool
fn max(self: Self) -> Option<Self::Item> where Self::Item: Ord
fn min(self: Self) -> Option<Self::Item> where Self::Item: Ord
fn sum<S>(self: Self) -> S where S: Sum<Self::Item>
fn product<P>(self: Self) -> P where P: Product<Self::Item>
```

## 🎯 Integration Strategy

### Phase 1: Core Traits (Priority 1.1)

- `ops` - Operator overloading
- `marker` - Send/Sync/Copy traits
- `convert` - Type conversion
- `default` - Default values
- `hash` - Hashing infrastructure

### Phase 2: Memory Management (Priority 1.2)

- `alloc` - Box, Rc, Arc
- Extend existing collections with Rust-style APIs

### Phase 3: Advanced Features (Priority 1.3)

- `iter` - Iterator system
- `regex` - Regular expressions
- `panic` - Panic handling
- `backtrace` - Stack traces

### Phase 4: System Integration (Priority 1.4)

- `process` - Process management
- `thread` - Threading utilities

## ⚠️ Language Feature Requirements

- **Trait System Extensions:** Associated types, generic traits
- **Advanced Generics:** Higher-kinded types for iterators
- **Lifetime System:** For Rc/Arc borrow checking
- **Move Semantics:** For ownership transfer
- **Pattern Matching:** For Result/Option handling

## 📋 Missing Native C Code Files

Based on detailed analysis of vex-runtime/c/ directory:

### Required New C Files:

1. **vex_hash.c** - SipHasher and other hash implementations
2. **vex_smartptr.c** - Rc and Arc reference counting
3. **vex_vecdeque.c** - Double-ended queue implementation
4. **vex_linkedlist.c** - Doubly-linked list implementation
5. **vex_backtrace.c** - Stack trace capture and printing

### Existing C Files (No new files needed):

- **vex_swisstable.c** - HashMap implementation (already exists)
- **vex_regex.c** - Regex engine (already exists)
- **vex_cmd.c** - Process management (already exists)
- **vex_testing.c** - Threading utilities (already exists)
- **vex_box.c** - Box implementation (already exists)
- **vex_panic functions** - Distributed across multiple files (already exists)

### Integration Notes:

- HashMap uses swisstable implementation (4x faster than Rust std)
- Threading functions exist in vex_testing.c and vex_cmd.c
- Panic handling is implemented via vex_panic() calls in various files
- Regex is fully implemented in vex_regex.c

## 📋 Missing Components

### Missing Native C Code Files:

1. **vex_hash.c** - SipHasher and hash utilities
2. **vex_smartptr.c** - Rc/Arc reference counting
3. **vex_vecdeque.c** - VecDeque implementation
4. **vex_linkedlist.c** - LinkedList implementation
5. **vex_backtrace.c** - Stack trace functionality

### Missing Rust Language Features:

- **Macros:** Declarative macros (derive, etc.)
- **Async/Await:** Built-in async support
- **Unsafe Code:** Unsafe blocks and raw pointers
- **FFI:** extern blocks for C interop
- **Procedural Macros:** Compile-time code generation
- **Specialization:** Trait specialization
- **GATs:** Generic associated types

## 🚀 Implementation Notes

1. **Trait Integration:** Vex'in mevcut trait sistemini genişlet
2. **Ownership Model:** Rust'ın ownership modelini Vex borrow checker'ına entegre et
3. **Zero-Cost:** Iterator adaptors compile-time optimization
4. **Safety:** Memory safety guarantees koru
5. **Performance:** Rust seviyesinde performans hedefle

---

_This document extends the Go-based planning with Rust stdlib features that enhance Vex's systems programming capabilities._
