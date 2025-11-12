# Go-Style Parameter Grouping

**Feature:** Vex v0.2.0  
**Status:** ✅ Production Ready  
**Date:** November 12, 2025

---

## Overview

Vex supports Go-style parameter grouping, allowing consecutive parameters of the same type to share a single type annotation.

## Syntax

```vex
// Traditional syntax (still supported)
fn add(a: i32, b: i32, c: i32): i32 {
    return a + b + c;
}

// Go-style grouping (new!)
fn add(a, b, c: i32): i32 {
    return a + b + c;
}
```

Both syntaxes are equivalent and produce identical AST nodes.

---

## Examples

### Simple Grouping

```vex
fn add3(a, b, c: i32): i32 {
    return a + b + c;
}
```

### Multiple Groups

```vex
fn process(x, y, z: f64, name, tag: string): void {
    println("Sum: ", x + y + z);
    println("Tags: ", name, tag);
}
```

### Mixed Parameters

```vex
fn compute(a, b: i32, factor: f64, c, d: i32): f64 {
    let sum = a + b + c + d;
    return i32_to_f64(sum) * factor;
}
```

### Struct Methods

```vex
struct Point {
    x: f64,
    y: f64,
    
    // Grouping works in methods too!
    distance_to(x1, y1: f64): f64 {
        let dx = self.x - x1;
        let dy = self.y - y1;
        return sqrt(dx * dx + dy * dy);
    }
    
    translate(dx, dy: f64)! {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
}
```

### Contract Definitions

```vex
contract Geometry {
    // Works in contracts too
    distance(x1, y1, x2, y2: f64): f64;
    translate(dx, dy: f64)!;
}
```

---

## Implementation Details

### Parser

The parser automatically expands grouped parameters during parsing:

```rust
// Input: (a, b, c: i32)
// Parsed as:
vec![
    Param { name: "a", ty: I32 },
    Param { name: "b", ty: I32 },
    Param { name: "c", ty: I32 },
]
```

### AST Representation

Each parameter gets its own `Param` node in the AST. The grouping syntax is purely a parsing convenience - the compiler sees fully expanded parameters.

### Type Checking

No changes needed. Since parameters are expanded during parsing, the type checker works unchanged.

---

## Benefits

✅ **Less repetition** - Reduces boilerplate for functions with many same-typed parameters  
✅ **Cleaner signatures** - Easier to read function declarations  
✅ **Go familiarity** - Go developers will find this natural  
✅ **Optional** - Traditional syntax still works  
✅ **Zero overhead** - Purely syntactic sugar, no runtime cost

---

## Comparison with Other Languages

### Go
```go
func add(a, b, c int) int {
    return a + b + c
}
```

### Vex
```vex
fn add(a, b, c: i32): i32 {
    return a + b + c;
}
```

### Rust (no grouping)
```rust
fn add(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}
```

---

## Testing

Comprehensive test suite: `examples/test_param_grouping.vx`

```bash
vex run examples/test_param_grouping.vx
```

Tests cover:
- Simple grouping (3 params)
- Multiple groups (2 groups)
- Mixed parameters (grouped + individual)
- Struct methods with grouping
- Contract definitions with grouping

---

## Future Considerations

Potential enhancements (not yet implemented):

1. **Default values with grouping**
   ```vex
   fn create(x, y, z: f64 = 0.0): Point3D { }
   ```

2. **Variadic parameters**
   ```vex
   fn sum(values...: i32): i32 { }
   ```

---

## Migration Guide

No migration needed! This is a purely additive feature. Existing code continues to work.

**Before (still works):**
```vex
fn distance(x1: f64, y1: f64, x2: f64, y2: f64): f64 {
    return sqrt((x2-x1)^2 + (y2-y1)^2);
}
```

**After (optional improvement):**
```vex
fn distance(x1, y1, x2, y2: f64): f64 {
    return sqrt((x2-x1)^2 + (y2-y1)^2);
}
```

Both versions compile to identical code.

---

## Related Documentation

- [VEX_IDENTITY.md](../VEX_IDENTITY.md) - Language identity and philosophy
- [REFERENCE.md](REFERENCE.md) - Complete syntax reference
- [examples/test_param_grouping.vx](../examples/test_param_grouping.vx) - Test suite
