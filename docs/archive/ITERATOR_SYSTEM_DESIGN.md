# Iterator System Design - Builtin Trait

**Date:** November 5, 2025  
**Version:** Vex v0.2.0  
**Status:** Planning (Part of Builtin Types - Phase 4)

---

## 🎯 Overview

Iterator is a **builtin trait** with special compiler support for `for-in` loops. Like Rust, all collections implement Iterator.

---

## 📐 Architecture

### Layer 1: Iterator Trait (Builtin)

```vex
// Builtin trait - available everywhere
trait Iterator<T> {
    fn next(&self!): Option<T>;

    // Default methods (implemented by compiler)
    fn map<U, F: Callable(T): U>(self, f: F): Map<Self, F> {
        Map { iter: self, f }
    }

    fn filter<F: Callable(T): bool>(self, f: F): Filter<Self, F> {
        Filter { iter: self, f }
    }

    fn collect<C: FromIterator<T>>(self): C {
        C::from_iter(self)
    }

    fn fold<B>(self, init: B, f: fn(B, T): B): B {
        let! acc = init;
        while let Some(item) = self.next() {
            acc = f(acc, item);
        }
        acc
    }
}
```

---

## 🔧 Implementation for Builtin Types

### Vec<T>::iter()

```vex
impl<T> Vec<T> {
    fn iter(&self): VecIter<T> {
        VecIter {
            vec: self,
            index: 0,
        }
    }
}

struct VecIter<T> {
    vec: &Vec<T>,
    index: usize,
}

impl<T> Iterator<T> for VecIter<T> {
    fn next(&self!): Option<&T> {
        if self.index < self.vec.len() {
            let item = &self.vec[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
```

### Map<K,V>::iter()

```vex
impl<K, V> Map<K, V> {
    fn iter(&self): MapIter<K, V> {
        MapIter {
            map: self,
            index: 0,
        }
    }

    fn keys(&self): MapKeys<K, V> {
        MapKeys { map: self, index: 0 }
    }

    fn values(&self): MapValues<K, V> {
        MapValues { map: self, index: 0 }
    }
}

struct MapIter<K, V> {
    map: &Map<K, V>,
    index: usize,
}

impl<K, V> Iterator<(K, V)> for MapIter<K, V> {
    fn next(&self!): Option<(&K, &V)> {
        // Call vex_map_iter_next()
    }
}
```

### Set<T>::iter()

```vex
impl<T> Set<T> {
    fn iter(&self): SetIter<T> {
        SetIter {
            set: self,
            index: 0,
        }
    }
}

struct SetIter<T> {
    set: &Set<T>,
    index: usize,
}

impl<T> Iterator<T> for SetIter<T> {
    fn next(&self!): Option<&T> {
        // Call vex_set_iter_next()
    }
}
```

---

## 🔄 for-in Loop Desugaring

### Source Code

```vex
let numbers = Vec::from([1, 2, 3, 4, 5]);
for num in numbers {
    println(num);
}
```

### Desugared (by compiler)

```vex
let numbers = Vec::from([1, 2, 3, 4, 5]);
{
    let! iter = numbers.iter();
    loop {
        match iter.next() {
            Some(num) => {
                println(num);
            }
            None => break,
        }
    }
}
```

**Compiler responsibilities:**

1. Call `.iter()` on collection
2. Generate loop with `iter.next()`
3. Pattern match on `Option<T>`
4. Bind variable in loop body

---

## 🚀 Iterator Adapters (Zero-Cost)

### Map<I, F>

```vex
struct Map<I, F> {
    iter: I,
    f: F,
}

impl<T, U, I: Iterator<T>, F: Callable(T): U> Iterator<U> for Map<I, F> {
    fn next(&self!): Option<U> {
        match self.iter.next() {
            Some(x) => Some(self.f(x)),
            None => None,
        }
    }
}
```

### Filter<I, F>

```vex
struct Filter<I, F> {
    iter: I,
    f: F,
}

impl<T, I: Iterator<T>, F: Callable(T): bool> Iterator<T> for Filter<I, F> {
    fn next(&self!): Option<T> {
        loop {
            match self.iter.next() {
                Some(x) => {
                    if self.f(x) {
                        return Some(x);
                    }
                }
                None => return None,
            }
        }
    }
}
```

### Take<I>

```vex
struct Take<I> {
    iter: I,
    n: usize,
    count: usize,
}

impl<T, I: Iterator<T>> Iterator<T> for Take<I> {
    fn next(&self!): Option<T> {
        if self.count < self.n {
            self.count += 1;
            self.iter.next()
        } else {
            None
        }
    }
}
```

---

## 📊 Usage Examples

### Basic Iteration

```vex
let numbers = Vec::from([1, 2, 3, 4, 5]);

// for-in (syntactic sugar)
for num in numbers {
    println(num);
}

// Manual iteration
let! iter = numbers.iter();
while let Some(num) = iter.next() {
    println(num);
}
```

### Chaining Adapters

```vex
let numbers = Vec::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

// Filter even numbers, double them, take first 3
let result = numbers.iter()
    .filter(|x| x % 2 == 0)  // [2, 4, 6, 8, 10]
    .map(|x| x * 2)           // [4, 8, 12, 16, 20]
    .take(3)                  // [4, 8, 12]
    .collect();               // Vec<i32>

// result = Vec::from([4, 8, 12])
```

### Map Iteration

```vex
let! scores = Map::new();
scores.insert("Alice", 100);
scores.insert("Bob", 85);
scores.insert("Charlie", 90);

// Iterate over key-value pairs
for (name, score) in scores {
    println(name, ": ", score);
}

// Iterate over keys only
for name in scores.keys() {
    println(name);
}

// Iterate over values only
for score in scores.values() {
    println(score);
}
```

### Set Iteration

```vex
let! tags = Set::new();
tags.insert("rust");
tags.insert("vex");
tags.insert("llvm");

for tag in tags {
    println(tag);
}
```

### Fold/Reduce

```vex
let numbers = Vec::from([1, 2, 3, 4, 5]);

// Sum all numbers
let sum = numbers.iter().fold(0, |acc, x| acc + x);
// sum = 15

// Product
let product = numbers.iter().fold(1, |acc, x| acc * x);
// product = 120
```

### Custom Iterator

```vex
struct Range {
    start: i32,
    end: i32,
}

impl Iterator<i32> for Range {
    fn next(&self!): Option<i32> {
        if self.start < self.end {
            let val = self.start;
            self.start += 1;
            Some(val)
        } else {
            None
        }
    }
}

// Usage
let range = Range { start: 0, end: 5 };
for i in range {
    println(i);  // 0, 1, 2, 3, 4
}
```

---

## 🔧 C Runtime Implementation

### Iterator State

```c
// vex-runtime/c/vex_iterator.c

typedef struct {
    void *collection;     // Pointer to Vec, Map, Set
    size_t index;         // Current position
    size_t elem_size;     // Element size
    void *(*next)(void*); // Next function pointer
    void (*free)(void*);  // Cleanup
} vex_iterator_t;

// Vec iterator
vex_iterator_t vex_vec_iter(vex_vec_t *vec) {
    return (vex_iterator_t){
        .collection = vec,
        .index = 0,
        .elem_size = vec->elem_size,
        .next = (void*(*)(void*))vex_vec_iter_next,
        .free = NULL,
    };
}

void *vex_vec_iter_next(vex_iterator_t *iter) {
    vex_vec_t *vec = (vex_vec_t*)iter->collection;
    if (iter->index < vec->len) {
        void *elem = (uint8_t*)vec->data + (iter->index * vec->elem_size);
        iter->index++;
        return elem;  // Zero-copy: return pointer
    }
    return NULL;  // End of iteration
}

// Map iterator (returns key-value pairs)
vex_iterator_t vex_map_iter(vex_map_t *map) {
    // Initialize SwissTable iterator
    return (vex_iterator_t){
        .collection = map,
        .index = 0,
        .elem_size = map->key_size + map->val_size,
        .next = (void*(*)(void*))vex_map_iter_next,
        .free = NULL,
    };
}

void *vex_map_iter_next(vex_iterator_t *iter) {
    // Call SwissTable next function
    // Returns pointer to (key, value) pair or NULL
}
```

---

## 📐 LLVM Codegen

### for-in Loop Codegen

```rust
// vex-compiler/src/codegen_ast/builtin_types/iterator.rs

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile for-in loop
    pub fn compile_for_in_loop(
        &mut self,
        var: &str,
        collection: &Expression,
        body: &Block,
    ) -> Result<(), String> {
        // 1. Compile collection
        let coll_val = self.compile_expression(collection)?;

        // 2. Call .iter() method
        let iter_val = self.compile_method_call(
            coll_val,
            "iter",
            &[],
        )?;

        // 3. Create loop blocks
        let loop_head = self.context.append_basic_block(self.current_function.unwrap(), "for_head");
        let loop_body = self.context.append_basic_block(self.current_function.unwrap(), "for_body");
        let loop_exit = self.context.append_basic_block(self.current_function.unwrap(), "for_exit");

        // 4. Jump to loop head
        self.builder.build_unconditional_branch(loop_head)?;

        // 5. Loop head: call iter.next()
        self.builder.position_at_end(loop_head);
        let next_val = self.compile_method_call(
            iter_val,
            "next",
            &[],
        )?;

        // 6. Pattern match on Option<T>
        let tag = self.builder.build_extract_value(next_val.into_struct_value(), 0, "tag")?;
        let is_some = self.builder.build_int_compare(
            IntPredicate::EQ,
            tag.into_int_value(),
            self.context.i8_type().const_int(1, false),
            "is_some"
        )?;

        self.builder.build_conditional_branch(is_some, loop_body, loop_exit)?;

        // 7. Loop body: extract value, bind variable, execute statements
        self.builder.position_at_end(loop_body);
        let value = self.builder.build_extract_value(next_val.into_struct_value(), 1, "value")?;
        self.insert_variable(var, value);

        for stmt in &body.statements {
            self.compile_statement(stmt)?;
        }

        self.builder.build_unconditional_branch(loop_head)?;

        // 8. Exit
        self.builder.position_at_end(loop_exit);

        Ok(())
    }
}
```

---

## 🎯 Zero-Cost Guarantees

### Inlining

All iterator methods are small and will be inlined:

```llvm
; Vec::iter().map().filter() chain
; After inlining and optimization:
define void @example() {
    ; Single loop, no function calls
    ; Conditions fused into one check
    ; No heap allocations
}
```

### Monomorphization

Each `Iterator<T>` is specialized per type:

- `VecIter<i32>` ≠ `VecIter<String>`
- No vtables, no dynamic dispatch

### Benchmarks

**Target:** Same performance as hand-written loops

```vex
// Iterator chain
let sum = numbers.iter()
    .filter(|x| x % 2 == 0)
    .map(|x| x * 2)
    .fold(0, |acc, x| acc + x);

// Equivalent manual loop
let! sum = 0;
for i in 0..numbers.len() {
    let x = numbers[i];
    if x % 2 == 0 {
        sum += x * 2;
    }
}
```

**Expected:** 0% overhead (both compile to identical LLVM IR)

---

## 📋 Implementation Checklist

### C Runtime

- [ ] `vex_iterator.c` - Generic iterator state
- [ ] `vex_vec_iter()` - Vec iterator
- [ ] `vex_map_iter()` - Map iterator
- [ ] `vex_set_iter()` - Set iterator

### AST & Parser

- [ ] `Type::Iterator(Box<Type>)` - Iterator type
- [ ] Parse `for item in collection` syntax
- [ ] Parse `.iter()`, `.map()`, `.filter()` method chains

### Codegen

- [ ] `builtin_types/iterator.rs` - Iterator trait codegen
- [ ] `compile_for_in_loop()` - Desugar for-in to while-let
- [ ] Iterator adapter structs (Map, Filter, Take)
- [ ] Inline all iterator methods

### Tests

- [ ] `examples/10_builtins/iterator_basic.vx`
- [ ] `examples/10_builtins/for_in_loop.vx`
- [ ] `examples/10_builtins/iterator_chain.vx`
- [ ] `examples/benchmarks/iterator_vs_manual.vx`

---

## 🚀 Timeline

**Part of Phase 4: Map, Set & Iterator (3-4 days)**

- **Day 1:** C runtime (vex_iterator.c, vex_vec_iter, vex_map_iter)
- **Day 2:** AST/Parser (Type::Iterator, for-in syntax)
- **Day 3:** Codegen (iterator trait, for-in desugaring)
- **Day 4:** Adapters (map, filter, fold) + tests

**Integration:** Works with existing Vec, Map, Set from Phase 4

---

**Decision:** Implement Iterator in Phase 4 alongside Map/Set for complete collections API.
