# std.collections

**High-performance collections for Vex**

Zero-cost wrappers over world-class C implementations.

## Features

- ✅ **HashMap<K, V>** - Generic hash map

  - Based on Google Swiss Tables algorithm (V1)
  - Fast insertions/lookups with SIMD optimization
  - NEON on ARM, AVX2 on x86
  - WyHash for fast hashing
  - Production-ready implementation

- ✅ **HashSet<T>** - Generic hash set
  - Built on HashMap internally
  - Ensures uniqueness
  - Same performance characteristics

## Usage

### HashMap

```vex
import { HashMap } from "std.collections";

let! scores = HashMap.new();

// Insert
scores.insert("Alice", 100);
scores.insert("Bob", 85);

// Get
match scores.get("Alice") {
    Option.Some(score) => println("Score: {}", score),
    Option.None => println("Not found"),
}

// Update
scores.insert("Alice", 110);  // Returns Option.Some(100)

// Contains
if scores.contains_key("Bob") {
    println("Bob is in the map");
}

// Remove
scores.remove("Bob");

// Length
println("Size: {}", scores.len());

// Clear
scores.clear();
```

### HashSet

```vex
import { HashSet } from "std.collections";

let! unique = HashSet.new();

// Insert
unique.insert(42);
unique.insert(100);
unique.insert(42);  // Ignored (already exists)

// Contains
if unique.contains(42) {
    println("Found!");
}

// Remove
unique.remove(100);

// Length
println("Unique items: {}", unique.len());
```

## API Reference

### HashMap<K, V>

#### Constructors

- `fn new(): HashMap<K, V>` - Create with default capacity (16)
- `fn with_capacity(cap: i64): HashMap<K, V>` - Create with specific capacity

#### Methods

- `fn insert(&self!, key: K, value: V): Option<V>` - Insert/update key-value
- `fn get(&self, key: K): Option<V>` - Get value by key
- `fn remove(&self!, key: K): Option<V>` - Remove key-value
- `fn contains_key(&self, key: K): bool` - Check if key exists
- `fn len(&self): i64` - Get number of entries
- `fn is_empty(&self): bool` - Check if empty
- `fn clear(&self!)` - Remove all entries

### HashSet<T>

#### Constructors

- `fn new(): HashSet<T>` - Create with default capacity
- `fn with_capacity(cap: i64): HashSet<T>` - Create with specific capacity

#### Methods

- `fn insert(&self!, value: T): bool` - Insert value (returns true if new)
- `fn contains(&self, value: T): bool` - Check if value exists
- `fn remove(&self!, value: T): bool` - Remove value
- `fn len(&self): i64` - Get number of elements
- `fn is_empty(&self): bool` - Check if empty
- `fn clear(&self!)` - Remove all elements

## Performance

Based on Swiss Tables V1 implementation (Apple Silicon M1):

| Operation | Performance | Notes                    |
| --------- | ----------- | ------------------------ |
| Insert    | ~20M ops/s  | Production-ready         |
| Lookup    | ~30M ops/s  | SIMD-optimized (NEON)    |
| Delete    | ~15M ops/s  | Efficient slot reuse     |

_Note: V2/V3 implementations planned for future performance improvements_

## Implementation

- **Native Code**: `vex-runtime/c/swisstable/vex_swisstable.c` (V1)
- **Vex Wrapper**: `src/hashmap.vx`, `src/hashset.vx`
- **Zero-cost**: Thin inline wrappers over C functions
- **SIMD**: Automatic NEON/AVX2 detection and usage

## Tests

```bash
# Run HashMap tests
vex run tests/hashmap_test.vx

# Run HashSet tests
vex run tests/hashset_test.vx

# Run examples
vex run examples/usage.vx
```

## Examples

See `examples/usage.vx` for:

- Basic HashMap usage
- Word frequency counter
- HashSet uniqueness
- Membership testing
- Performance demo (1000 items)

## License

MIT - Part of Vex standard library
