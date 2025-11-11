# Error Handling

**Version:** 0.1.2
**Last Updated:** November 2025

This document describes Vex's error handling system, including the `Result<T, E>` and `Option<T>` types, pattern matching for error handling, and best practices.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1
7. \1

---

## Overview

Vex provides two primary types for handling potential failure:

- **`Option<T>`**: Represents a value that may or may not exist
- **`Result<T, E>`**: Represents either success (`T`) or failure (`E`)

Both types encourage explicit handling of potential errors rather than exceptions or null pointers.

---

## Option<T> Type

The `Option<T>` type represents an optional value. It has two variants:

``````vex
enum Option<T> {
    Some(T),
    None
}
```

### Basic Usage

[14 lines code: ```vex]

### Common Methods

[17 lines code: ```vex]

---

## Result<T, E> Type

The `Result<T, E>` type represents either success or failure. It has two variants:

``````vex
enum Result<T, E> {
    Ok(T),
    Err(E)
}
```

### Basic Usage

[14 lines code: ```vex]

### Common Methods

[20 lines code: ```vex]

---

## Pattern Matching for Errors

Pattern matching provides elegant error handling:

[26 lines code: ```vex]

### Nested Matching

[21 lines code: ```vex]

---

## Error Propagation

Vex supports the `?` operator for concise error propagation, similar to Rust.

**Implementation Status**: ✅ **COMPLETE** (v0.1.2)

### The `?` Operator

The question mark operator (`?`) provides automatic error propagation for `Result<T, E>` types:

[11 lines code: ```vex]

**How it works**:

The `?` operator desugars to a match expression:

``````vex
// This code:
let result = divide(10, 2)?;

// Becomes:
let result = match divide(10, 2) {
    Ok(v) => v,
    Err(e) => return Err(e)
};
```

### Nested Error Propagation

[25 lines code: ```vex]

### Chain of Operations

``````vex
fn nested_operations(): Result<i32, string> {
    let x = divide(10, 2)?;   // Ok(5)
    let y = divide(x, 0)?;    // Err("Division by zero") - propagates here
    let z = divide(y, 2)?;    // Never reached
    return Ok(z);
}
```

**Implementation Details**:

- **Parser**: `vex-parser/src/parser/operators.rs` - Parses `expr?` syntax
- **AST**: `vex-ast/src/lib.rs` - `Expression::QuestionMark(Box<Expression>)`
- **Codegen**: `vex-compiler/src/codegen_ast/expressions/mod.rs` - Desugars to Result match
- **Test file**: `examples/test_question_mark.vx`

### Early Returns

[16 lines code: ```vex]

---

## Custom Error Types

Define custom error types using enums:

[24 lines code: ```vex]

### Error Traits

Implement common error traits for better ergonomics:

[13 lines code: ```vex]

---

## Best Practices

### 1. Use Result for Operations That Can Fail

[9 lines code: ```vex]

### 2. Define Specific Error Types

[11 lines code: ```vex]

### 3. Use ? for Early Returns

``````vex
// Good: Clear control flow
fn process_request(req: Request): Result<Response, Error> {
    let user = authenticate(req)?;
    let data = validate_input(req)?;
    let result = perform_business_logic(user, data)?;

    Ok(create_response(result))
}
```

### 4. Handle Errors at Appropriate Levels

[9 lines code: ```vex]

### 5. Avoid unwrap() in Production

``````vex
// Bad: Will panic in production
let value = result.unwrap();

// Good: Handle the error
let value = match result {
    Ok(v) => v,
    Err(e) => return Err(e) // or provide default
};
```

### 6. Use expect() for Programming Errors

``````vex
// Good: Document assumptions
let config = load_config().expect("Config file should always exist");

// Bad: Silent unwrap
let config = load_config().unwrap();
```

---

**Previous**: \1 
**Next**: \1

**Maintained by**: Vex Language Team</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/Specifications/17_Error_Handling.md
