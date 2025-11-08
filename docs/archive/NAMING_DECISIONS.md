# Naming Decisions - Collections & Types

**Date:** November 5, 2025  
**Decision:** Simplified naming for builtin types

---

## 🎯 Core Principle

**Hashing is the default implementation** - no need for "Hash" prefix.

---

## ✅ Builtin Types (No Imports)

| Type           | Description        | Implementation          |
| -------------- | ------------------ | ----------------------- |
| `Map<K, V>`    | Hash map           | SwissTable (Google)     |
| `Set<T>`       | Hash set           | Wrapper over Map<T, ()> |
| `Vec<T>`       | Dynamic array      | Growable buffer         |
| `Option<T>`    | Nullable type      | Enum with Some/None     |
| `Result<T, E>` | Error handling     | Enum with Ok/Err        |
| `String`       | UTF-8 string       | Owned, growable         |
| `Iterator<T>`  | Iteration protocol | Trait + for-in syntax   |
| `Channel<T>`   | Message passing    | Lock-free MPSC          |

**Why Map/Set (not HashMap/HashSet)?**

- Simpler, cleaner names
- Hashing is the obvious default choice
- Follows precedent: JavaScript (`Map`, `Set`), Python (`dict`, `set`)
- When you need sorted collections, use `TreeMap`/`TreeSet` (explicit)

---

## 📚 Standard Library (Explicit Imports)

| Type            | Description         | Implementation       |
| --------------- | ------------------- | -------------------- |
| `TreeMap<K, V>` | Sorted map          | B-Tree               |
| `TreeSet<T>`    | Sorted set          | Wrapper over TreeMap |
| `LinkedList<T>` | Doubly-linked       | Pointer-based        |
| `Deque<T>`      | Double-ended queue  | Ring buffer          |
| `Arc<T>`        | Atomic ref counting | Thread-safe          |
| `Rc<T>`         | Reference counting  | Single-threaded      |
| `Mutex<T>`      | Mutual exclusion    | OS locks             |
| `RwLock<T>`     | Read-write lock     | OS locks             |

**Why TreeMap/TreeSet (not BTreeMap)?**

- Consistent with Map/Set naming
- "Tree" clearly indicates sorted/ordered behavior
- Implementation detail (B-Tree) is less important than semantics

---

## 🔄 Comparison with Other Languages

### Rust

```rust
HashMap<K, V>  // Explicit "Hash" prefix
BTreeMap<K, V> // Explicit "BTree" prefix
HashSet<T>
BTreeSet<T>
```

### Vex (Simplified)

```vex
Map<K, V>      // Hash is default
TreeMap<K, V>  // Explicit "Tree" for sorted
Set<T>         // Hash is default
TreeSet<T>     // Explicit "Tree" for sorted
```

**Rationale:**

- 99% of use cases want hash-based collections (O(1) average)
- Sorted collections (O(log n)) are used less frequently
- When you need sorting, you explicitly ask for it: `TreeMap`

### JavaScript

```javascript
Map; // Hash-based
Set; // Hash-based
```

✅ Vex follows this pattern!

### Python

```python
dict  # Hash-based
set   # Hash-based
```

✅ Similar philosophy

---

## 📝 Usage Examples

### Builtin Types (No Imports)

```vex
// Map - hash-based by default
let! scores = Map::new();
scores.insert("Alice", 100);
scores.insert("Bob", 85);

if let Some(score) = scores.get("Alice") {
    println("Alice: ", score);
}

// Set - hash-based by default
let! tags = Set::new();
tags.insert("rust");
tags.insert("vex");
tags.insert("llvm");

if tags.contains("rust") {
    println("Found Rust!");
}

// Iterator - for-in loops
let numbers = Vec::from([1, 2, 3, 4, 5]);
for num in numbers {
    println(num);
}

// Iterator chaining
let doubled = numbers.iter()
    .filter(|x| x % 2 == 0)
    .map(|x| x * 2)
    .collect();
```

### Standard Library (Explicit Import)

```vex
import std.collections.TreeMap;
import std.collections.TreeSet;

// TreeMap - sorted by key
let! scores = TreeMap::new();
scores.insert("Charlie", 90);
scores.insert("Alice", 100);
scores.insert("Bob", 85);

// Iteration is in sorted order!
for (name, score) in scores {
    println(name, ": ", score);
}
// Output: Alice: 100, Bob: 85, Charlie: 90

// TreeSet - sorted elements
let! nums = TreeSet::new();
nums.insert(5);
nums.insert(1);
nums.insert(3);

for num in nums {
    println(num);
}
// Output: 1, 3, 5
```

---

## 🔧 C Runtime Naming

### Builtin Types

```c
vex_map.c       // Map<K,V> (hash-based, SwissTable)
vex_set.c       // Set<T> (wrapper over Map)
vex_vec.c       // Vec<T>
vex_string.c    // String
vex_iterator.c  // Iterator<T>
vex_channel.c   // Channel<T>
```

### Standard Library

```c
vex_tree_map.c  // TreeMap<K,V> (B-Tree)
vex_tree_set.c  // TreeSet<T>
vex_deque.c     // Deque<T>
```

**Consistency:** File names match type names

---

## 📐 AST Type Enum

```rust
// vex-ast/src/lib.rs
pub enum Type {
    // Builtin types
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Vec(Box<Type>),
    Map(Box<Type>, Box<Type>),     // Hash-based
    Set(Box<Type>),                 // Hash-based
    Iterator(Box<Type>),
    Channel(Box<Type>),

    // Standard library types (parsed as Named)
    Named(String),  // TreeMap, TreeSet, Deque, Arc, Rc, etc.
}
```

---

## 🎯 Decision Summary

1. ✅ **Map/Set** (not HashMap/HashSet) - Hashing is default
2. ✅ **TreeMap/TreeSet** (not BTreeMap/BTreeSet) - "Tree" indicates sorting
3. ✅ **Iterator** (builtin trait) - for-in loop support
4. ✅ **Simpler names** - Less typing, cleaner code
5. ✅ **Explicit when needed** - Import TreeMap when you need sorting

---

## 📚 Related Documents

- `BUILTIN_TYPES_ARCHITECTURE.md` - Full builtin types design
- `ITERATOR_SYSTEM_DESIGN.md` - Iterator trait and for-in loops
- `VEX_RUNTIME_STDLIB_ROADMAP.md` - Implementation timeline
- `TODO.md` - Phase 4 tasks updated

---

**Approved:** November 5, 2025  
**Rationale:** Simplicity, consistency with JavaScript/Python, pragmatic defaults
