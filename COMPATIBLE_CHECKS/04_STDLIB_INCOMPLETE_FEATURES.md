# 04: Standard Library Incomplete Features

**Severity:** 🟡 HIGH  
**Category:** Standard Library / API Completeness  
**Analysis Date:** 15 Kasım 2025  
**Status:** IDENTIFIED - FEATURE GAPS

---

## Executive Summary

Vex stdlib **26 eksik feature** var. Bazı modules minimal API expose ediyor, test coverage %30 civarı, documentation eksik. Go/Rust stdlib seviyesine ulaşmak için ciddi genişleme gerekiyor.

**Ana Sorunlar:**
- Collections (Vec, HashMap, etc.) incomplete
- io module println crash ediyor
- time module struct ABI broken
- net/http yok
- json/yaml/toml parser yok
- regex yok

**Impact:** Real-world apps yazılamıyor, third-party dependencies şart.

---

## Critical Issues (🔴)

### Issue 1: io.println Crash

**File:** `vex-libs/std/io/src/lib.vx`  
**Severity:** 🔴 CRITICAL  
**Impact:** Basic output broken

**Evidence:**
```bash
$ vex run examples/test_io.vx
Segmentation fault: 11
```

**Problem:**
```vex
import io;

fn main() {
    io.println("Hello");  // ❌ Crash
}
```

**Root Cause:** Runtime `println` implementation dereferences null pointer

**Recommendation:**
```rust
// Fix in vex-runtime/src/io/mod.rs
#[no_mangle]
pub extern "C" fn vex_println(s: *const u8, len: usize) {
    if s.is_null() {  // Add null check
        eprintln!("Error: null string passed to println");
        return;
    }
    
    let slice = unsafe { std::slice::from_raw_parts(s, len) };
    if let Ok(string) = std::str::from_utf8(slice) {
        println!("{}", string);
    }
}
```

**Effort:** 1 day

---

### Issue 2: Collections API Minimal

**File:** `vex-libs/std/collections/src/vec.vx`  
**Severity:** 🔴 CRITICAL  
**Impact:** Cannot work with vectors properly

**Evidence:**
```vex
// vex-libs/std/collections/src/vec.vx
contract Vec<T> {
    fn new() -> Vec<T>;
    fn push(self!, item: T);
    fn len(self) -> usize;
    // ❌ Missing: pop, get, insert, remove, clear, contains, etc.
}
```

**Missing Methods:**
- `pop() -> Option<T>`
- `get(index: usize) -> Option<&T>`
- `insert(index: usize, item: T)`
- `remove(index: usize) -> T`
- `clear()`
- `contains(&T) -> bool`
- `iter() -> Iterator<T>`
- `map<U>(fn(T) -> U) -> Vec<U>`
- `filter(fn(&T) -> bool) -> Vec<T>`
- `sort()`, `reverse()`, `dedup()`

**Recommendation:**
```vex
// Add to vec.vx
contract Vec<T> {
    // Existing
    fn new() -> Vec<T>;
    fn push(self!, item: T);
    fn len(self) -> usize;
    
    // NEW
    fn pop(self!) -> Option<T>;
    fn get(self, index: usize) -> Option<&T>;
    fn get!(self!, index: usize) -> Option<&T!>;
    fn insert(self!, index: usize, item: T);
    fn remove(self!, index: usize) -> T;
    fn clear(self!);
    fn contains(self, item: &T) -> bool where T: Eq;
    fn iter(self) -> VecIter<T>;
    fn map<U>(self, f: fn(T) -> U) -> Vec<U>;
    fn filter(self, f: fn(&T) -> bool) -> Vec<T>;
    fn sort(self!) where T: Ord;
    fn reverse(self!);
    fn dedup(self!) where T: Eq;
    fn capacity(self) -> usize;
    fn reserve(self!, additional: usize);
    fn shrink_to_fit(self!);
}
```

**Effort:** 2-3 weeks

---

### Issue 3: HashMap Missing

**File:** `vex-libs/std/collections/` (does not exist)  
**Severity:** 🔴 CRITICAL  
**Impact:** No key-value data structure

**Problem:**
```vex
// This doesn't exist:
import collections.HashMap;

fn main() {
    let map = HashMap<String, i32>.new();
    map.insert("age", 30);
    // ❌ Not implemented
}
```

**Recommendation:**
```vex
// Create vex-libs/std/collections/src/hashmap.vx
contract HashMap<K, V> where K: Hash + Eq {
    fn new() -> HashMap<K, V>;
    fn with_capacity(capacity: usize) -> HashMap<K, V>;
    fn insert(self!, key: K, value: V) -> Option<V>;
    fn get(self, key: &K) -> Option<&V>;
    fn get!(self!, key: &K) -> Option<&V!>;
    fn remove(self!, key: &K) -> Option<V>;
    fn contains_key(self, key: &K) -> bool;
    fn len(self) -> usize;
    fn is_empty(self) -> bool;
    fn clear(self!);
    fn keys(self) -> Keys<K>;
    fn values(self) -> Values<V>;
    fn iter(self) -> Iter<K, V>;
}
```

**Implementation:** Use FNV hash or SipHash, open addressing or chaining

**Effort:** 3-4 weeks

---

### Issue 4: String API Incomplete

**File:** `vex-libs/std/string/src/lib.vx`  
**Severity:** 🔴 CRITICAL  
**Impact:** Cannot manipulate strings

**Current API:**
```vex
contract String {
    fn new() -> String;
    fn from(s: &str) -> String;
    fn len(self) -> usize;
    // ❌ Missing: split, trim, replace, contains, etc.
}
```

**Missing Methods:**
- `split(separator: &str) -> Vec<String>`
- `trim() -> String`
- `trim_start()`, `trim_end()`
- `contains(needle: &str) -> bool`
- `starts_with(prefix: &str) -> bool`
- `ends_with(suffix: &str) -> bool`
- `replace(from: &str, to: &str) -> String`
- `to_uppercase()`, `to_lowercase()`
- `substring(start: usize, end: usize) -> String`
- `chars() -> CharIter`
- `bytes() -> ByteIter`

**Effort:** 2 weeks

---

### Issue 5: Option/Result Combinators Missing

**File:** `vex-libs/std/option/src/lib.vx`  
**Severity:** 🔴 CRITICAL  
**Impact:** Cannot chain Option/Result operations

**Current:**
```vex
contract Option<T> {
    Some(T),
    None
}
// ❌ No methods at all!
```

**Recommendation:**
```vex
contract Option<T> {
    Some(T),
    None,
    
    fn is_some(self) -> bool;
    fn is_none(self) -> bool;
    fn unwrap(self) -> T;
    fn unwrap_or(self, default: T) -> T;
    fn unwrap_or_else(self, f: fn() -> T) -> T;
    fn expect(self, msg: &str) -> T;
    fn map<U>(self, f: fn(T) -> U) -> Option<U>;
    fn and_then<U>(self, f: fn(T) -> Option<U>) -> Option<U>;
    fn or(self, other: Option<T>) -> Option<T>;
    fn or_else(self, f: fn() -> Option<T>) -> Option<T>;
    fn filter(self, predicate: fn(&T) -> bool) -> Option<T>;
}

contract Result<T, E> {
    Ok(T),
    Err(E),
    
    fn is_ok(self) -> bool;
    fn is_err(self) -> bool;
    fn unwrap(self) -> T;
    fn unwrap_err(self) -> E;
    fn expect(self, msg: &str) -> T;
    fn map<U>(self, f: fn(T) -> U) -> Result<U, E>;
    fn map_err<F>(self, f: fn(E) -> F) -> Result<T, F>;
    fn and_then<U>(self, f: fn(T) -> Result<U, E>) -> Result<U, E>;
    fn or(self, other: Result<T, E>) -> Result<T, E>;
}
```

**Effort:** 1-2 weeks

---

## High Priority Issues (🟡)

### Issue 6: Iterator Trait Missing

**File:** N/A  
**Severity:** 🟡 HIGH  
**Impact:** Cannot iterate collections idiomatically

**Recommendation:**
```vex
contract Iterator<T> {
    type Item;
    fn next(self!) -> Option<Self.Item>;
    
    fn map<U>(self, f: fn(T) -> U) -> Map<Self, U>;
    fn filter(self, f: fn(&T) -> bool) -> Filter<Self>;
    fn fold<B>(self, init: B, f: fn(B, T) -> B) -> B;
    fn collect<B>(self) -> B where B: FromIterator<T>;
    fn count(self) -> usize;
    fn sum(self) -> T where T: Add;
}
```

**Effort:** 3-4 weeks

---

### Issue 7: File API Minimal

**File:** `vex-libs/std/fs/src/lib.vx`  
**Severity:** 🟡 HIGH  
**Impact:** Cannot work with files properly

**Current:**
```vex
fn read_to_string(path: &str) -> Result<String, Error>;
fn write(path: &str, contents: &str) -> Result<(), Error>;
// ❌ Missing: metadata, permissions, directories, etc.
```

**Missing:**
- `File.open(path)`, `File.create(path)`
- `File.read(buffer)`, `File.write(data)`
- `File.seek(pos)`, `File.tell()`
- `metadata(path)`, `size()`, `modified_time()`
- `create_dir(path)`, `remove_dir(path)`
- `read_dir(path) -> Iterator<DirEntry>`
- `copy(from, to)`, `rename(from, to)`
- `permissions(path)`, `set_permissions(path, perms)`

**Effort:** 2-3 weeks

---

### Issue 8: Error Handling Inconsistent

**Severity:** 🟡 HIGH  
**Impact:** Different modules use different error types

**Problem:**
```vex
// fs uses Error
fn read_to_string(path: &str) -> Result<String, Error>;

// strconv uses bool
fn parse_int(s: &str) -> (i64, bool);

// Should standardize on Result<T, E>
```

**Effort:** 1 week

---

### Issue 9: JSON Parser Missing

**File:** N/A  
**Severity:** 🟡 HIGH  
**Impact:** Cannot parse/serialize JSON

**Recommendation:**
```vex
// vex-libs/std/json/
contract Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>)
}

fn parse(s: &str) -> Result<Value, ParseError>;
fn stringify(v: &Value) -> String;
```

**Effort:** 3-4 weeks

---

### Issue 10: Network Stack Missing

**File:** N/A  
**Severity:** 🟡 HIGH  
**Impact:** Cannot make HTTP requests, no server

**Recommendation:**
```vex
// vex-libs/std/net/
contract TcpListener {
    fn bind(addr: &str) -> Result<TcpListener, Error>;
    fn accept(self!) -> Result<TcpStream, Error>;
}

contract TcpStream {
    fn connect(addr: &str) -> Result<TcpStream, Error>;
    fn read(self!, buf: &[u8]!) -> Result<usize, Error>;
    fn write(self!, buf: &[u8]) -> Result<usize, Error>;
}

// vex-libs/std/http/
contract Client {
    fn get(url: &str) -> Result<Response, Error>;
    fn post(url: &str, body: &str) -> Result<Response, Error>;
}

contract Server {
    fn new(addr: &str) -> Server;
    fn route(self!, path: &str, handler: Handler);
    fn run(self);
}
```

**Effort:** 6-8 weeks (major project)

---

### Issue 11: time Module Broken

**File:** `vex-libs/std/time/src/lib.vx`  
**Severity:** 🟡 HIGH  
**Impact:** Struct ABI issues (see Issue #1 in 03_CODEGEN)

**Problem:**
```vex
contract Duration {
    secs: i64,
    nanos: i32
}

fn now() -> Instant;  // ❌ Crashes due to ABI mismatch
```

**Effort:** Fix after codegen struct ABI fixed

---

### Issue 12: Regex Missing

**Severity:** 🟡 HIGH  
**Impact:** Cannot do pattern matching

**Recommendation:**
```vex
// vex-libs/std/regex/
contract Regex {
    fn new(pattern: &str) -> Result<Regex, Error>;
    fn is_match(self, text: &str) -> bool;
    fn find(self, text: &str) -> Option<Match>;
    fn find_all(self, text: &str) -> Iterator<Match>;
    fn replace(self, text: &str, rep: &str) -> String;
}
```

**Effort:** 4-5 weeks (or wrap oniguruma/PCRE2)

---

### Issue 13: Path API Missing

**Severity:** 🟡 HIGH  
**Impact:** Cannot manipulate file paths

**Recommendation:**
```vex
contract Path {
    fn new(s: &str) -> Path;
    fn join(self, other: &Path) -> Path;
    fn parent(self) -> Option<&Path>;
    fn file_name(self) -> Option<&str>;
    fn extension(self) -> Option<&str>;
    fn is_absolute(self) -> bool;
    fn is_relative(self) -> bool;
}
```

**Effort:** 1-2 weeks

---

## Medium Priority Issues (🟢)

### Issue 14: fmt Module Missing

**Severity:** 🟢 MEDIUM  
**Impact:** Cannot format strings

**Recommendation:**
```vex
fn format(template: &str, args: &[Any]) -> String;
// format("Hello {}", &["world"])
```

**Effort:** 2-3 weeks

---

### Issue 15: env Module Limited

**File:** `vex-libs/std/env/src/lib.vx`  
**Severity:** 🟢 MEDIUM  
**Impact:** Only get/set env vars

**Missing:**
- `current_dir()`, `set_current_dir(path)`
- `home_dir()`, `temp_dir()`
- `args() -> Vec<String>`

**Effort:** 1 week

---

### Issue 16: process Module Limited

**Severity:** 🟢 MEDIUM  
**Impact:** Cannot spawn child processes

**Missing:**
- `Command.new(program)`
- `Command.arg(arg)`, `Command.args(args)`
- `Command.spawn()`, `Command.output()`
- `Child.wait()`, `Child.kill()`

**Effort:** 2 weeks

---

### Issue 17: crypto Module Missing

**Severity:** 🟢 MEDIUM  
**Impact:** Cannot hash, encrypt

**Recommendation:**
```vex
// vex-libs/std/crypto/
fn sha256(data: &[u8]) -> [u8; 32];
fn md5(data: &[u8]) -> [u8; 16];
// AES, RSA, etc.
```

**Effort:** 3-4 weeks

---

### Issue 18: rand Module Missing

**Severity:** 🟢 MEDIUM  
**Impact:** Cannot generate random numbers

**Effort:** 1-2 weeks

---

### Issue 19: sync Module Incomplete

**Severity:** 🟢 MEDIUM  
**Impact:** No Mutex, RwLock, Arc

**Effort:** 2-3 weeks

---

### Issue 20: Test Framework Missing

**Severity:** 🟢 MEDIUM  
**Impact:** Cannot write unit tests

**Recommendation:**
```vex
#[test]
fn test_addition() {
    assert_eq(2 + 2, 4);
}
```

**Effort:** 2-3 weeks

---

## Low Priority Issues (🔵)

### Issue 21: Documentation Comments

**Severity:** 🔵 LOW  
**Impact:** No doc generation

**Effort:** 1 week

---

## Metrics Summary

| Category | Critical | High | Medium | Low | Total |
|----------|----------|------|--------|-----|-------|
| Collections | 3 | 1 | 0 | 0 | 4 |
| I/O | 1 | 2 | 1 | 0 | 4 |
| String/Text | 1 | 2 | 1 | 0 | 4 |
| Network | 0 | 2 | 0 | 0 | 2 |
| Error Handling | 1 | 1 | 0 | 0 | 2 |
| System | 0 | 2 | 3 | 0 | 5 |
| Testing/Docs | 0 | 0 | 1 | 1 | 2 |
| **TOTAL** | **5** | **10** | **6** | **1** | **26** |

---

## Test Coverage Analysis

```bash
$ ./test_stdlib_modules.sh
✅ cmd: OK (exit 0)
✅ fs: OK (exit 0)
✅ strconv: OK (exit 0)
✅ env: OK (exit 0)
✅ process: OK (exit 0)
⚠️ time: Partial (exit 0 but limited API)
⚠️ memory: Import works (no tests)
❌ io: CRASH (segfault)
❌ collections: Minimal
❌ net: Does not exist
❌ json: Does not exist
❌ regex: Does not exist

Overall: 5/13 modules working (~38%)
```

---

## Implementation Roadmap

### Phase 1: Fix Critical (Week 1-2)
- [ ] Fix io.println crash
- [ ] Expand Vec API (pop, get, insert, etc.)
- [ ] Add HashMap
- [ ] Expand String API
- [ ] Add Option/Result methods

### Phase 2: High Priority (Week 3-6)
- [ ] Iterator trait
- [ ] File API expansion
- [ ] JSON parser
- [ ] Error handling standardization
- [ ] Path API

### Phase 3: Network (Week 7-14)
- [ ] TCP/UDP sockets
- [ ] HTTP client
- [ ] HTTP server

### Phase 4: Medium Priority (Week 15-20)
- [ ] fmt module
- [ ] Expand env/process
- [ ] crypto module
- [ ] rand module
- [ ] sync primitives

---

## Testing Plan

```vex
// test_collections.vx
import collections.{Vec, HashMap};

fn test_vec() {
    let v = Vec<i32>.new();
    v.push(1);
    v.push(2);
    assert_eq(v.len(), 2);
    assert_eq(v.pop(), Some(2));
    assert_eq(v.get(0), Some(&1));
}

fn test_hashmap() {
    let m = HashMap<String, i32>.new();
    m.insert("age", 30);
    assert_eq(m.get("age"), Some(&30));
    assert(m.contains_key("age"));
}

// test_io.vx
import io;

fn test_println() {
    io.println("Hello");  // Should not crash
    io.print("World");
}

// test_json.vx
import json;

fn test_parse() {
    let s = r#"{"name": "Alice", "age": 30}"#;
    let v = json.parse(s).unwrap();
    // ...
}
```

---

## Related Issues

- [03_CODEGEN_LLVM_ISSUES.md](./03_CODEGEN_LLVM_ISSUES.md) - Struct ABI breaks time module
- [05_RUNTIME_FFI_PROBLEMS.md](./05_RUNTIME_FFI_PROBLEMS.md) - FFI affects I/O, net modules

---

## References

- Rust std: https://doc.rust-lang.org/std/
- Go stdlib: https://pkg.go.dev/std
- Python stdlib: https://docs.python.org/3/library/

---

**Next Steps:** Fix io.println crash immediately, then expand Vec/HashMap APIs.
