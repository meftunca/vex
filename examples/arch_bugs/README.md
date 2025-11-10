# Architecture Bugs & Solutions

This directory contains minimal reproducible examples for critical architecture bugs and their solutions.

## Bug #1: Generic Methods in Imported Modules Not Registered

**Problem**: When a module is imported, generic methods defined with Golang-style syntax are not registered in the compiler's function_defs.

**Symptom**:

```vex
// lib.vx
fn (self: &Container<T>!) insert(item: T) { ... }

// main.vx
import { Container } from "lib.vx";
let! c = Container<i32>.new();
c.insert(42);  // ❌ Error: Method 'insert' not found
```

**Root Cause**:

- Import resolution compiles modules separately
- Golang-style methods (`fn (self: &Type) method()`) are parsed as top-level functions with receivers
- These functions are added to the imported module's scope but not propagated to the main program
- `compile_program()` only sees the main file's items (10 items in hashmap_test.vx)
- HashMap's methods are in hashmap.vx but never reach main program's function_defs

**Solution**: See `generic_methods_import/` test case
