# Phase 4: Lifetime Analysis - Implementation Summary

**Date**: 4 Kasım 2025  
**Status**: ✅ COMPLETED (Basic Implementation)  
**Files Modified**: 4 files  
**Tests Added**: 3 new tests  
**Duration**: ~3 hours

## 🎯 Objectives

Implement Phase 4 of the borrow checker to prevent dangling references by tracking variable lifetimes and validating that references don't outlive their referents.

## 📋 Implementation Details

### New Module: `lifetimes.rs` (464 lines)

**Location**: `vex-compiler/src/borrow_checker/lifetimes.rs`

**Core Data Structures**:

```rust
pub struct LifetimeChecker {
    variable_scopes: HashMap<String, usize>,  // Maps variable names to scope depth
    current_scope: usize,                     // Current scope depth (0=global, 1=params, 2+=locals)
    references: HashMap<String, String>,      // Maps reference variables to their referents
    in_scope: HashSet<String>,                // Currently accessible variables
}
```

**Key Features**:

- Scope-based lifetime tracking with depth levels:
  - **Scope 0**: Global (functions, constants - always valid)
  - **Scope 1**: Function parameters (valid to return)
  - **Scope 2+**: Local variables (invalid to return references)
- Return statement validation for dangling references
- Recursive expression and statement checking
- Integration with existing borrow checker phases

### Error Types Added (errors.rs)

```rust
DanglingReference { reference: String, referent: String }
UseAfterScopeEnd { variable: String }
ReturnDanglingReference { variable: String }
```

Each with helpful Display implementations showing error context and hints.

### Integration (mod.rs)

Added `LifetimeChecker` to the `BorrowChecker` orchestrator:

```rust
pub struct BorrowChecker {
    immutability: ImmutabilityChecker,
    moves: MoveChecker,
    borrows: BorrowRulesChecker,
    lifetimes: LifetimeChecker,  // NEW
}
```

Sequential execution: Phase 1 → Phase 2 → Phase 3 → **Phase 4** ✅

### CLI Fix (vex-cli/src/main.rs)

**Critical Bug Fixed**: Borrow checker was only running for `vex compile`, not `vex run`.

**Before**:

```rust
Commands::Run { ... } => {
    // Parse
    let ast = parser.parse_file()?;
    // Codegen (NO BORROW CHECKER!)
    let codegen = ASTCodeGen::new(...);
}
```

**After**:

```rust
Commands::Run { ... } => {
    // Parse
    let ast = parser.parse_file()?;

    // Run borrow checker ✅
    println!("   🔍 Running borrow checker...");
    let mut borrow_checker = vex_compiler::BorrowChecker::new();
    if let Err(e) = borrow_checker.check_program(&ast) {
        anyhow::bail!("⚠️  Borrow checker error: {}", e);
    }
    println!("   ✅ Borrow check passed");

    // Codegen
    let codegen = ASTCodeGen::new(...);
}
```

## 🧪 Tests

### Test 1: `10_lifetime_return_local.vx` (Should Fail ❌)

```vex
fn get_reference(): &i32 {
    let x = 42;
    return &x;  // ERROR: x is local (scope 2), will be dropped
}
```

**Output**:

```
Error: ⚠️  Borrow checker error: cannot return reference to local variable `x`
the variable will be dropped at the end of the function
help: consider returning an owned value or accepting a reference parameter
```

✅ **Result**: Correctly caught dangling reference

### Test 2: `11_lifetime_return_param.vx` (Should Pass ✅)

```vex
fn get_ref(x: &i32): &i32 {
    return x;  // OK: x is a parameter (scope 1), safe to return
}

fn main(): i32 {
    let value = 42;
    let ref_val = get_ref(&value);
    return 0;
}
```

**Output**:

```
🚀 Running: "examples/00_borrow_checker/11_lifetime_return_param.vx"
   ✅ Parsed 11_lifetime_return_param successfully
   🔍 Running borrow checker...
   ✅ Borrow check passed
```

✅ **Result**: Correctly allowed valid reference return

### Test 3: `12_lifetime_scope_end.vx` (Documentation)

Documents limitation: Flow-sensitive analysis (tracking references through assignments) not yet implemented.

## 🔧 Technical Challenges Solved

### Challenge 1: Function Scope Management

**Problem**: Parameters and local variables both declared in function, but only locals should error on return.

**Solution**: Double scope entry in `check_function()`:

```rust
fn check_function(&mut self, func: &Function) -> BorrowResult<()> {
    self.enter_scope();  // Scope 1: Parameters
    for param in &func.params {
        self.declare_variable(&param.name);
    }

    self.enter_scope();  // Scope 2: Function body (locals)
    self.check_block(&func.body)?;

    self.exit_scope();  // Exit body
    self.exit_scope();  // Exit function
    Ok(())
}
```

### Challenge 2: Global Item Registration

**Problem**: Functions and constants weren't registered at global scope, causing `UseAfterScopeEnd` errors.

**Solution**: Register all global items in `check_item()`:

```rust
fn check_item(&mut self, item: &Item) -> BorrowResult<()> {
    match item {
        Item::Function(func) => {
            self.variable_scopes.insert(func.name.clone(), 0);  // Global scope
            self.in_scope.insert(func.name.clone());            // Always accessible
            self.check_function(func)
        }
        Item::Const(const_def) => {
            self.variable_scopes.insert(const_def.name.clone(), 0);
            self.in_scope.insert(const_def.name.clone());
            Ok(())
        }
        _ => Ok(()),
    }
}
```

### Challenge 3: CLI Integration

**Problem**: Borrow checker code existed but never ran during `vex run`.

**Solution**: Added borrow checker invocation to both `Commands::Compile` and `Commands::Run` paths. This fix revealed that Phase 4 was working correctly - it just wasn't being called!

## 📊 Results

**Before Phase 4**:

- 41/43 tests passing (95.3%)
- Borrow checker: 17 tests (Phases 1-3)
- No lifetime analysis

**After Phase 4**:

- 44/46 tests passing (95.7%)
- Borrow checker: 20 tests (Phases 1-4) ✅
- Basic lifetime analysis operational

## 🚧 Limitations & Future Work

### Current Limitations

1. **Flow-sensitive analysis not implemented**:

   ```vex
   let! ref_val = &temp;
   {
       let local = 42;
       ref_val = &local;  // Should error but doesn't
   }
   // ref_val now dangling
   ```

2. **Only return statements validated**: Assignments and passing to functions not yet tracked.

3. **No cross-function analysis**: Each function checked in isolation.

### Future Enhancements (Phase 4.1)

1. **Assignment tracking**: Validate reference assignments in nested scopes
2. **Control flow analysis**: Track lifetimes through if/match/loops
3. **Function argument validation**: Ensure references passed to functions are valid
4. **Struct field lifetimes**: Track references stored in structs
5. **Lifetime parameters**: Support explicit lifetime annotations (Rust-style `'a`)

### Estimated Effort

- **Phase 4.1** (Flow-sensitive analysis): 3-4 days
- **Phase 4.2** (Struct lifetimes): 2-3 days
- **Phase 4.3** (Lifetime parameters): 4-5 days

## 📝 Code Changes Summary

**Files Modified**:

1. `vex-compiler/src/borrow_checker/lifetimes.rs` - NEW (464 lines)
2. `vex-compiler/src/borrow_checker/errors.rs` - Added 3 error types
3. `vex-compiler/src/borrow_checker/mod.rs` - Integrated Phase 4
4. `vex-cli/src/main.rs` - Fixed borrow checker execution

**Tests Added**:

1. `examples/00_borrow_checker/10_lifetime_return_local.vx`
2. `examples/00_borrow_checker/11_lifetime_return_param.vx`
3. `examples/00_borrow_checker/12_lifetime_scope_end.vx`

**Documentation Updated**:

1. `TODO.md` - Phase 4 marked complete, test counts updated

## 🎉 Conclusion

Phase 4 basic implementation is complete and functional. The lifetime checker successfully prevents the most common source of dangling references: returning references to local variables. This brings Vex one step closer to Rust-level memory safety.

**Next Steps**: Choose between:

1. **Option A**: Continue with flow-sensitive analysis (Phase 4.1)
2. **Option B**: Move to next high-priority feature (data-carrying enums or default trait methods)
3. **Option C**: Implement dynamic dispatch for trait objects

---

**Commit Message**:

```
feat(borrow-checker): Implement Phase 4 lifetime analysis

- Add LifetimeChecker module with scope-based tracking
- Prevent returning references to local variables
- Fix CLI to run borrow checker on `vex run` command
- Add 3 new lifetime validation tests (20 borrow checker tests total)
- Update test count: 44/46 passing (95.7%)

BREAKING: Borrow checker now runs on all code execution paths
```
