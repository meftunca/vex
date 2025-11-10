# Vex Language Tests TODO

This file tracks the implementation of comprehensive tests for Vex language features based on docs/REFERENCE.md.

## Language Features to Test

- [x] **Lexical Structure**

  - Comments (line and block)
  - Keywords (all categories)
  - Identifiers (valid and invalid)
  - Operators (all types)

- [x] **Type System**

  - Primitive Types (integers, floats, bool, string, etc.)
  - Compound Types (arrays, tuples, slices)
  - Advanced Types (union types)
  - Type Aliases
  - Reflection Builtins (typeof, type_id, etc.)

- [x] **Variables and Constants**

  - Immutable variables (let)
  - Mutable variables (let!)
  - Type annotations
  - Shadowing
  - Constants (const)

- [x] **Functions and Methods**

  - Function syntax (basic, entry point)
  - Method mutability (inline and external styles)
  - Method calls (mutable and immutable)

- [x] **Control Flow**

  - If/Elif/Else statements
  - Match expressions (literals, enums, tuples, or patterns)
  - For loops (C-style)
  - While loops
  - Defer statements
  - Break/Continue

- [x] **Structs**

  - Basic struct syntax
  - Struct instantiation
  - Struct tags (Go-style backticks)
  - Generic structs

- [x] **Enums**

  - Unit variants
  - Discriminant values
  - Tuple variants
  - Builtin enums (Option, Result)

- [x] **Traits**

  - Trait definition
  - Trait implementation (struct impl Trait)
  - Default methods
  - Required method enforcement

- [x] **Generics**

  - Generic functions
  - Generic structs
  - Generic enums
  - Type constraints (future)

- [x] **Pattern Matching**

  - Match expressions
  - Pattern types (literals, enums, tuples, or patterns)

- [x] **Error Handling**

  - Option<T> usage
  - Result<T, E> usage
  - Error propagation (?)

- [x] **Concurrency**

  - Goroutines (go keyword)
  - Async/Await
  - Channels
  - GPU computing (gpu, launch)

- [x] **Memory Management**

  - Ownership
  - Borrowing (immutable and mutable references)
  - Borrowing rules
  - Raw pointers (unsafe)

- [x] **Modules and Imports**

  - Import statements (namespace, named)
  - Export declarations
  - Nested modules

- [x] **Policy System**

  - Policy declaration
  - Policy inheritance (with keyword)
  - Struct application
  - Inline metadata

- [x] **Operators**
  - Operator overloading (trait-based)
  - Arithmetic operators
  - Comparison operators
  - Logical operators
  - Bitwise operators
  - Assignment operators
  - Member access (dot operator)
  - Range operators

## Implementation Notes

- Each feature should have its own subdirectory under vex-lang-tests/
- Tests should be comprehensive, covering edge cases and error conditions
- Use Vex syntax v0.1.2 (no ::, no mut, no ->, etc.)
- Focus on language features only, not standard library
- Tests should be runnable with `vex run test_file.vx`
