# Vex Stdlib Planning - 03: Collections and Algorithms

**Priority:** 3
**Status:** Partial (collections exists with Map/Set, sort and advanced containers missing)
**Dependencies:** builtin, unsafe

## 📦 Packages in This Category

### 3.1 container
**Status:** ❌ Missing (Go has container/list, container/ring, etc.)
**Description:** Additional container data structures

#### Current Implementation
- Basic Map and Set exist in collections module
- Vec<T> exists in core module
- No advanced containers like LinkedList, Ring, Heap

#### Required Subpackages

#### container/list
```vex
struct Element<T> {
    next: *Element<T>,
    prev: *Element<T>,
    list: *List<T>,
    value: T,
}

struct List<T> {
    root: Element<T>,
    len: usize,
}

fn new<T>(): List<T>
fn len(l: &List<T>): usize
fn front(l: &List<T>): *Element<T>
fn back(l: &List<T>): *Element<T>
fn push_front(l: &mut List<T>, v: T): *Element<T>
fn push_back(l: &mut List<T>, v: T): *Element<T>
fn insert_before(l: &mut List<T>, v: T, mark: *Element<T>): *Element<T>
fn insert_after(l: &mut List<T>, v: T, mark: *Element<T>): *Element<T>
fn remove(l: &mut List<T>, e: *Element<T>): T
```

#### container/ring
```vex
struct Ring<T> {
    next: *Ring<T>,
    prev: *Ring<T>,
    value: T,
}

fn new<T>(n: usize): *Ring<T>
fn len(r: *Ring<T>): usize
fn next(r: *Ring<T>): *Ring<T>
fn prev(r: *Ring<T>): *Ring<T>
fn move(r: *Ring<T>, n: usize): *Ring<T>
fn link(s: *Ring<T>, t: *Ring<T>)
fn unlink(r: *Ring<T>, n: usize): *Ring<T>
```

#### container/heap
```vex
trait Interface<T> {
    fn len(self: &Self): usize
    fn less(self: &Self, i: usize, j: usize): bool
    fn swap(self: &mut Self, i: usize, j: usize)
    fn push(self: &mut Self, x: T)
    fn pop(self: &mut Self): T
}

fn init<T, I: Interface<T>>(h: &mut I)
fn push<T, I: Interface<T>>(h: &mut I, x: T)
fn pop<T, I: Interface<T>>(h: &mut I): T
fn remove<T, I: Interface<T>>(h: &mut I, i: usize): T
fn fix<T, I: Interface<T>>(h: &mut I, i: usize)
```

#### Dependencies
- builtin

### 3.2 sort
**Status:** ❌ Missing (critical for collections)
**Description:** Sorting and searching algorithms

#### Required Functions
```vex
// Slice sorting
fn sort<T: Ord>(data: &mut [T])
fn sort_stable<T: Ord>(data: &mut [T])
fn is_sorted<T: Ord>(data: &[T]): bool

// Custom sorting
fn sort_by<T, F>(data: &mut [T], less: F) where F: fn(&T, &T): bool
fn sort_stable_by<T, F>(data: &mut [T], less: F) where F: fn(&T, &T): bool

// Searching
fn search<T: Ord>(data: &[T], x: T): usize
fn search_by<T, F>(data: &[T], f: F): usize where F: fn(&T): bool

// Introspective sort
fn introsort<T: Ord>(data: &mut [T])
fn heapsort<T: Ord>(data: &mut [T])
fn quicksort<T: Ord>(data: &mut [T])

// Utility functions
fn reverse<T>(data: &mut [T])
fn rotate<T>(data: &mut [T], mid: usize)
```

#### Dependencies
- builtin

#### Notes
- **Language Issue:** Trait bounds (`T: Ord`) support needed
- **Generics:** Complex generic constraints required

### 3.3 index
**Status:** ❌ Missing (Go has index/suffixarray)
**Description:** Index data structures

#### index/suffixarray
```vex
struct Index {
    data: []u8,
    sa: []i32,
}

fn new(data: []u8): Index
fn bytes(i: &Index): []u8
fn len(i: &Index): usize
fn lookup(i: &Index, s: []u8, n: usize): []i32
fn lookup_all(i: &Index, s: []u8): []i32
fn find_all_index(i: &Index, s: []u8, n: usize): [][]i32
```

#### Dependencies
- builtin
- sort

### 3.4 collections (extend existing)
**Status:** ✅ Exists (extend with new types)
**Description:** Extend existing collections package

#### Additional Types Needed
```vex
// Priority Queue
struct PriorityQueue<T> {
    heap: Vec<T>,
    less: fn(&T, &T): bool,
}

// Deque
struct Deque<T> {
    buf: Vec<T>,
    head: usize,
    tail: usize,
    count: usize,
}

// BitSet
struct BitSet {
    bits: Vec<usize>,
}
```

#### Additional Functions
```vex
// PriorityQueue
fn new_priority_queue<T>(less: fn(&T, &T): bool): PriorityQueue<T>
fn push<T>(pq: &mut PriorityQueue<T>, item: T)
fn pop<T>(pq: &mut PriorityQueue<T>): T
fn peek<T>(pq: &PriorityQueue<T>): &T
fn len<T>(pq: &PriorityQueue<T>): usize

// Deque
fn new_deque<T>(): Deque<T>
fn push_front<T>(d: &mut Deque<T>, item: T)
fn push_back<T>(d: &mut Deque<T>, item: T)
fn pop_front<T>(d: &mut Deque<T>): T
fn pop_back<T>(d: &mut Deque<T>): T

// BitSet
fn new_bitset(): BitSet
fn set(bs: &mut BitSet, i: usize)
fn clear(bs: &mut BitSet, i: usize)
fn is_set(bs: &BitSet, i: usize): bool
fn len(bs: &BitSet): usize
```

#### Dependencies
- builtin
- sort (for PriorityQueue)

## 🎯 Implementation Priority

1. **sort** - Essential for all collections
2. **collections extensions** - Extend existing Vec/Map/Set
3. **container/list** - Doubly-linked list
4. **container/heap** - Heap operations
5. **index/suffixarray** - Advanced indexing

## ⚠️ Language Feature Issues

- **Trait Bounds:** `T: Ord` syntax not confirmed
- **Function Types in Structs:** Storing function pointers in structs
- **Complex Generics:** Higher-order generic functions
- **Raw Pointers:** For linked list implementations

## 📋 Missing Critical Dependencies

- **Ord Trait:** Ordering trait for comparisons
- **Fn Traits:** Function traits for callbacks
- **Associated Types:** For advanced generic programming

## 🚀 Next Steps

1. Implement sort package with basic algorithms
2. Add Ord trait and comparison functions
3. Extend collections with PriorityQueue, Deque
4. Implement container/list
5. Add container/heap interface