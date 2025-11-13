# Operator Overloading

**Version:** 0.2.0  
**Date:** November 12, 2025  
**Status:** Draft

---

## 🎯 Core Principles

1. **Contract-based**: Operators MUST be declared in a contract first.
2. **`op` prefix mandatory**: All operator methods use `op+`, `op-`, `op*`, etc.
3. **Multiple overloads**: One contract can define multiple operator overloads with different parameter types.
4. **Explicit implementation**: No automatic derivation — the implementing type explicitly defines its operator methods.

---

## 📋 Overview

Operator overloading in Vex is implemented as a capability provided through contracts. Contracts declare `op` methods and implementing types (typically `struct`s) provide concrete methods to support operator syntax.

The operator names follow the form `op<token>`, for example `op+`, `op-`, `op==`, as method names in contract declarations and implementations.

The compiler resolves an operator expression `a + b` by:

1. Checking whether `a`'s type implements the contract that defines `op+`.
2. Looking up the overload that best matches the right-hand side argument (`b`), including builtin primitive rules and `Self` substitutions.
3. Compiling a direct call to the corresponding method (`a.op+(b)`).

---

## Syntax: Contract Declarations

```vex
contract Add {
    op+(rhs: Self): Self;                    // Same type add
    op+(rhs: i32): Self;                     // Overload with i32
    op+(rhs: f64): Self;                     // Overload with f64
}
```

Rules:

- Contract name describes the capability (`Add`, `Mul`, `Eq`).
- All operator methods must use `op` prefix.
- Overloads are allowed as distinct method signatures.
- Return type is explicit and independent per overload.

---

## Syntax: Implementation

```vex
struct Vec2 impl Add {
    x: f64,
    y: f64,
}

// Vec2 + Vec2
fn (self: Vec2) op+(other: Vec2): Vec2 {
    return Vec2 { x: self.x + other.x, y: self.y + other.y };
}

// Vec2 + scalar
fn (self: Vec2) op+(scalar: f64): Vec2 {
    return Vec2 { x: self.x + scalar, y: self.y + scalar };
}
```

Rules:

- Struct must declare `impl ContractName`.
- Implementations must match contract method signatures (including `!` for mutability when applicable).
- Multiple overloads are separate method definitions.

---

## Resolution & Semantics

- Operator expression `a + b` resolves to `a.op+(b)` if `a`'s type implements the contract with a matching signature.
- Matching uses compile-time types; no implicit conversions except for the builtin `extends` contract mapping for primitives.
- If no match is found on the left-hand type, the compiler attempts to check for `coerce` rules and builtin operators (primitives), and finally raises an error.

---

## Phase 1: Arithmetic Operators

Contracts and typical builtin extensions.

```vex
contract Add {
    op+(rhs: Self): Self;
}

contract Sub { op-(rhs: Self): Self; }
contract Mul { op*(rhs: Self): Self; }
contract Div { op/(rhs: Self): Self; }
contract Rem { op%(rhs: Self): Self; }

contract Neg { op-(): Self; }
```

Primitive types are declared to `extend` these contracts in the standard library (builtin extensions). The compiler emits the corresponding IR for primitive operations.

---

## Phase 1.5: Bitwise Operators

```vex
contract BitAnd { op&(rhs: Self): Self; }
contract BitOr  { op|(rhs: Self): Self; }
contract BitXor { op^(rhs: Self): Self; }
contract BitNot { op~(): Self; }
contract Shl    { op<<(rhs: i32): Self; }
contract Shr    { op>>(rhs: i32): Self; }
```

Builtins: integer types are `extends` these contracts.

---

## Phase 2: Comparison & Logical Operators

```vex
contract Eq  { op==(rhs: Self): bool; op!=(rhs: Self): bool; }
contract Ord { op<(rhs: Self): bool; op>(rhs: Self): bool; op<=(rhs: Self): bool; op>=(rhs: Self): bool; }
contract Not { op!(): bool; }
```

Rules and special behaviors:

- If `op==` is implemented and `op!=` is not present, the compiler auto-generates `op!=` as `!self.op==(rhs)`.
- `Ord` must implement all four methods (no auto-generation), maintaining consistent semantics.

---

## Phase 3: Compound Assignment & Index Operators

```vex
contract AddAssign { op+=(rhs: Self); }
contract SubAssign { op-=(rhs: Self); }
contract MulAssign { op*=(rhs: Self); }
contract DivAssign { op/=(rhs: Self); }
contract RemAssign { op%=(rhs: Self); }

contract Index    { type Output; op[](index: i32): Output; }
contract IndexMut { type Output; op[]=(index: i32, value: Output); }

// Bitwise assignment
contract BitAndAssign { op&=(rhs: Self); }
contract BitOrAssign  { op|=(rhs: Self); }
contract BitXorAssign { op^=(rhs: Self); }

contract ShlAssign { op<<=(rhs: i32); }
contract ShrAssign { op>>=(rhs: i32); }
```

- Compound assignment operators (`op+=`) typically mutate `self` and require `!` in implementations to indicate mutability for contracts where necessary.

---

## Mutability & Side Effects

- Operators that are mutation semantics (like `op+=`) should be defined to mutate `self` with signature including `!` where the contract or implementation requires a mutable receiver.
- The `!` syntax in method signatures indicates the method requires a mutable reference for `self`.

---

## Builtin Primitives & `extends`

- The standard library declares `extends` for primitive types and builtin contracts to provide default operator support. These are specified in `stdlib/core/builtin_contracts.vx` and related files.
- `extends` indicates a special compiler-provided mapping for primitives and is not the same as a user `impl` for user-defined types.

---

## Auto-Generation & Default Implementations

- The compiler may auto-generate comparisons (`op!=` from `op==`) when semantically correct.
- Default contract methods that have body can be used to offer generic operator compositions (e.g., `op!=` defaulting to `!op==`).

---

## Diagnostics & Errors

- Type mismatch in operator operands or return type mismatches result in compile errors.
- When multiple overloads are applicable, the compiler uses exact type match precedence, then consider generic matches and coercions.

---

## Examples

See `examples/` for several samples and tests:

- `examples/test_operator_overloading.vx`
- `examples/test_operator_unary_neg.vx`
- `examples/operator/05_all_operators.vx`
- `examples/PROPOSAL_operator_syntax.vx`

---

## Implementation Notes (Compiler)

- Parser: Operator method names are parsed as `op` token followed by punctuation (e.g., `+`, `-`, `==`).
- Resolution: Compiler looks up `impl` for contract, ensuring correct arity and parameter types.
- Codegen: For user-defined types, the compiler emits calls to the method implementation; for primitives, it emits optimized IR.

---

## Security & Safety Considerations

- Overloading assignment operators like `op=` is either forbidden or strongly discouraged due to move/copy semantics that can easily introduce surprising behavior.
- Implementations must be careful around mutability to avoid aliasing and data races in concurrent code.

---

## Future Work / Open Questions

- Cross-type operator inference rules and conversions.
- Automatic generation of `op!=` vs `op==` symmetries for compound or complex types.
- Partial ordering inference for `Ord`.

---

## Changelog

- v0.2.0: Specification draft consolidating operator overloading phases and builtin contracts.

---

> See also: `OPERATOR_OVERLOADING_SPEC.md` at repository root for the original in-depth draft and additional examples.
