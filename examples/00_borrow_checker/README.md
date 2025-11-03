# Borrow Checker Examples

This directory contains examples demonstrating Vex's compile-time borrow checker.

## Phase 1: Immutability Check (let vs let!)

### 01_immutability_error.vx

❌ **Should fail**: Attempts to assign to an immutable variable.

```vex
let x = 10;
x = 20;  // ERROR: cannot assign to immutable variable `x`
```

### 02_immutability_valid.vx

✅ **Should pass**: Properly uses mutable variables with `let!`.

```vex
let! y = 5;
y = 15;  // OK: y is mutable
```

## Phase 2: Move Semantics (Use-After-Move)

### 03_move_error.vx

❌ **Should fail**: Uses a value after it has been moved.

```vex
let s = "hello";
let s2 = s;  // s moves to s2
log(s);      // ERROR: use of moved value: `s`
```

### 04_move_test.vx

Tests both move and copy semantics - has intentional errors for testing.

### 05_move_valid.vx

✅ **Should pass**: Demonstrates proper move semantics and copy types.

```vex
let x = 42;
let y = x;   // Copy (i32 is Copy type)
log(x);      // OK: x is still valid

let s = "hello";
let s2 = s;  // Move
log(s2);     // OK: using moved value
```

## Testing

Run individual examples:

```bash
~/.cargo/target/release/vex compile examples/00_borrow_checker/01_immutability_error.vx
```

Expected outputs:

- `01_immutability_error.vx`: ⚠️ Borrow checker error
- `02_immutability_valid.vx`: ✅ Borrow check passed
- `03_move_error.vx`: ⚠️ Borrow checker error (use-after-move)
- `05_move_valid.vx`: ✅ Borrow check passed

## Phase 3: Borrow Rules (1 Mutable XOR N Immutable)

### 06-09: Borrow Rules Examples

⚠️ **Note**: Parser currently only supports `&x` (immutable reference).
`&x!` (mutable reference) syntax will be added in next phase.

- `06_multiple_mutable_error.vx`: Multiple immutable borrows (valid)
- `07_mutable_while_immutable_error.vx`: Reserved for &x! syntax
- `08_mutation_while_borrowed_error.vx`: Reserved for mutation checks
- `09_multiple_immutable_valid.vx`: Multiple immutable borrows (valid)

## Implementation Status

- ✅ **Phase 1**: Immutability Check (let vs let!)
- ✅ **Phase 2**: Move Semantics (use-after-move prevention)
- ✅ **Phase 3**: Borrow Rules (1 mutable XOR N immutable) - Logic ready, awaiting &x! syntax
- ⏳ **Phase 4**: Lifetime Analysis (scope-based lifetimes)
