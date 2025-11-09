# Attribute Removal - Complete Documentation

**Date:** November 9, 2025
**Status:** ✅ COMPLETE

## Decision

**Vex will NOT support Rust-style `#[attribute]` syntax.**

Rationale:

- Adds complexity without clear benefit
- Policy system already provides metadata mechanism
- Go-style struct tags (backticks) handle serialization needs
- LLVM attributes can be set directly in compiler
- Intrinsics use `@` prefix (`@vectorize`, `@gpu`)

## Changes Made

### 1. AST Changes (`vex-ast/src/lib.rs`)

**Removed:**

- `struct Attribute { name, args }`
- `enum AttributeArg { Single, KeyValue, List }`
- `Function.attributes: Vec<Attribute>`
- `ExternBlock.attributes: Vec<Attribute>`
- `ExternFunction.attributes: Vec<Attribute>`

### 2. Parser Changes

**Files Modified:**

- `vex-parser/src/parser/items/externs.rs` - Removed `attributes: Vec::new()`
- `vex-parser/src/parser/items/functions.rs` - Removed `attributes: Vec::new()`
- `vex-parser/src/parser/items/structs.rs` - Removed `attributes: Vec::new()`

### 3. Compiler Changes

**Files Modified:**

- `vex-compiler/src/codegen_ast/expressions/calls/method_calls.rs` - Removed attribute field
- `vex-compiler/src/codegen_ast/expressions/special/closures.rs` - Removed attribute field

### 4. Lexer Changes (`vex-lexer/src/lib.rs`)

**Removed:**

- `Token::Hash` - No longer needed (no `#[...]` syntax)

### 5. Documentation Updates

**Updated Files:**

- `Specifications/07_Structs_and_Data_Types.md` - Documented Go-style struct tags
- `Specifications/02_Lexical_Structure.md` - Removed `#` from operators table
- `Specifications/01_Introduction_and_Overview.md` - Changed "Attribute Macros" to "NOT IN VEX"
- `Specifications/21_Mutability_and_Pointers.md` - Removed `#[repr(C)]` example
- `docs/REFERENCE.md` - Removed `#[repr(C)]` example

**Note Added to All Specs:**

> Vex does NOT use Rust-style `#[attribute]` syntax. Attributes are not part of the language.

## Alternative Mechanisms

### 1. Struct Metadata: Go-Style Tags

```vex
struct User {
    id: u64        `json:"id" db:"pk"`,
    username: string `json:"username"`,
}
```

### 2. Compiler Intrinsics: `@` Prefix

```vex
@vectorize
fn compute(data: [f32]) { ... }

@gpu
fn parallel_sum(arr: [i32]) { ... }
```

### 3. Policy System: Metadata Annotations

```vex
policy Serializable {
    format: "json",
    skip_empty: true,
}

struct User with Serializable {
    name: string,
    age: i32,
}
```

## Test Results

- ✅ Build: SUCCESS (4.79s)
- ✅ Tests: 254/260 passing (97.7%)
- ✅ No regressions from attribute removal

## Benefits

1. **Simplicity**: Less syntax to learn
2. **Consistency**: One way to add metadata (struct tags)
3. **No Macro Hell**: Vex avoids Rust's complex macro system
4. **Clear Separation**: Intrinsics (`@`) vs metadata (backticks) vs policies
5. **Maintainability**: Less AST complexity, easier compiler maintenance

## Future

If metadata needs arise that can't be solved with existing mechanisms:

- Consider extending policy system
- Consider new intrinsic types
- DO NOT add `#[attribute]` syntax back

---

**This decision is final. No Rust-style attributes in Vex.**
