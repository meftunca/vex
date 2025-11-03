# Examples Reorganization Plan (v0.9)

## 📁 New Structure

```
examples/
├── 01_basics/           # Core language features
├── 02_functions/        # Functions, methods, recursion
├── 03_control_flow/     # If, switch, loops
├── 04_types/            # Structs, enums, tuples, aliases
├── 05_generics/         # Generic types and functions
├── 06_patterns/         # Pattern matching, destructuring
├── 07_strings/          # String operations, f-strings
├── 08_algorithms/       # Classic algorithms
└── README.md
```

## ✅ Keep & Migrate (Working Features)

### 01_basics/

- ✅ `hello.vx` → `01_basics/hello_world.vx`
- ✅ `simple_test.vx` → `01_basics/simple_return.vx`
- ✅ `test_let_mutable.vx` → `01_basics/let_mutable.vx` (v0.9 example!)
- 🆕 `01_basics/variables.vx` (let, let!, const examples)
- 🆕 `01_basics/types_basic.vx` (i32, f32, bool, string)
- 🆕 `01_basics/comments.vx`

### 02_functions/

- ✅ `calculator.vx` → `02_functions/basic.vx`
- ✅ `method_call_test.vx` → `02_functions/methods.vx`
- ✅ `struct_methods.vx` → `02_functions/struct_methods.vx`
- 🆕 `02_functions/recursion.vx` (fib + factorial combined)
- 🆕 `02_functions/parameters.vx` (multiple params, returns)

### 03_control_flow/

- ✅ `conditional_simple_test.vx` → `03_control_flow/if_else.vx`
- ✅ `switch_minimal.vx` → `03_control_flow/switch.vx`
- ✅ `match_simple.vx` → `03_control_flow/match_basic.vx`
- 🆕 `03_control_flow/loops.vx` (while, for examples)

### 04_types/

- ✅ `struct_literal_basic.vx` → `04_types/struct_basic.vx`
- ✅ `struct_test.vx` → `04_types/struct_advanced.vx`
- ✅ `enum_test.vx` → `04_types/enum_basic.vx`
- ✅ `enum_pattern_test.vx` → `04_types/enum_patterns.vx`
- ✅ `tuple_test.vx` → `04_types/tuple_basic.vx`
- ✅ `type_alias_test.vx` → `04_types/type_aliases.vx`
- 🆕 `04_types/references.vx` (&T, &T! examples)

### 05_generics/

- ✅ `generics_test.vx` → `05_generics/functions.vx`
- ✅ `interface_test.vx` → `05_generics/interfaces.vx`
- 🆕 `05_generics/structs.vx` (Option<T>, Result<T> examples)

### 06_patterns/

- ✅ `struct_pattern_simple.vx` → `06_patterns/struct_destructure.vx`
- ✅ `tuple_comprehensive.vx` → `06_patterns/tuple_destructure.vx`
- ✅ `enum_pattern_test.vx` → `06_patterns/enum_match.vx`

### 07_strings/

- ✅ `strings.vx` → `07_strings/literals.vx`
- ✅ `test_fstring_simple.vx` → `07_strings/formatting.vx`
- 🆕 `07_strings/operations.vx` (concat, slice, etc)

### 08_algorithms/

- ✅ `fibonacci.vx` → `08_algorithms/fibonacci.vx`
- ✅ `factorial.vx` → `08_algorithms/factorial.vx`
- ✅ `gcd.vx` → `08_algorithms/gcd.vx`
- ✅ `prime.vx` → `08_algorithms/prime_check.vx`
- ✅ `power.vx` → `08_algorithms/power.vx`
- ✅ `sum_array.vx` → `08_algorithms/sum_array.vx`

## ❌ Remove (Not Implemented / Broken)

### Async/Concurrency (Not implemented)

- ❌ `async_*.vx` (7 files) - Async/await not in compiler
- ❌ `concurrent_*.vx` (2 files) - Channels not implemented
- ❌ `go_*.vx` (2 files) - Go-style concurrency not implemented
- ❌ `pthread_test.vx` - Low-level threading

### GPU/SIMD (Not implemented)

- ❌ `gpu_*.vx` (3 files) - GPU kernels not implemented
- ❌ `simd_*.vx` (1 file) - SIMD not implemented

### FFI/External (Not ready)

- ❌ `ffi_*.vx` (4 files) - FFI partial, examples outdated
- ❌ `openssl_crypto_test.vx` - External library

### Advanced Features (Not implemented)

- ❌ `http_client.vx` - HTTP library not implemented
- ❌ `compression_benchmark.vx` - Compression not implemented
- ❌ `filesystem_*.vx` (2 files) - FS library not ready
- ❌ `std_time_test.vx` - Time library not ready
- ❌ `regex_test.vx` - Regex not implemented
- ❌ `error_handling.vx` - Result/Option not fully working

### Traits (Parser only)

- ❌ `trait_*.vx` (3 files) - Trait codegen not implemented

### Test Infrastructure

- ❌ `test_suite.vx` - Test framework not implemented
- ❌ `run_test.vx` - Test runner

### Duplicates/Outdated

- ❌ `new_syntax_v06.vx` - Outdated syntax version
- ❌ `simple_return.vx` - Duplicate of simple_test
- ❌ `no_import_test.vx` - Same as simple_test
- ❌ `with_imports.vx` - Imports not functional
- ❌ `import_test.vx` - Imports parse only
- ❌ `advanced_types.vx` - Conditional types not implemented
- ❌ `conditional_types_test.vx` - Not implemented
- ❌ `intersection_test.vx` - Parse only, no codegen
- ❌ `union_*.vx` (2 files) - Parse only, no codegen

### Debug/Temporary Files

- ❌ `enum_debug*.vx` (2 files) - Debug files
- ❌ `enum_constructor_test.vx` - Specific bug test
- ❌ `enum_data_test.vx` - Redundant
- ❌ `field_access_test.vx` - Basic feature
- ❌ `method_mutable_test.vx` - Covered in struct_methods
- ❌ `test_*.vx` (13 files) - Ad-hoc tests, need proper organization
- ❌ `try_simple.vx` - Try/catch not implemented
- ❌ `test_unwrap*.vx` (2 files) - Result unwrap not ready

## 📊 Summary

| Category     | Keep   | Remove | New   |
| ------------ | ------ | ------ | ----- |
| Basics       | 3      | 5      | 3     |
| Functions    | 3      | 0      | 2     |
| Control Flow | 3      | 0      | 1     |
| Types        | 6      | 8      | 1     |
| Generics     | 2      | 3      | 1     |
| Patterns     | 3      | 0      | 0     |
| Strings      | 2      | 0      | 1     |
| Algorithms   | 6      | 0      | 0     |
| **TOTAL**    | **28** | **55** | **9** |

## 🎯 Migration Steps

1. ✅ Create new directory structure
2. ✅ Migrate working examples with v0.9 syntax updates
3. ✅ Write new comprehensive examples
4. ✅ Update README.md with new structure
5. ✅ Delete outdated/non-working examples
6. ✅ Test all examples compile and run

## 🔄 v0.9 Syntax Updates Required

All examples need:

- ❌ `var x = 10;` → ✅ `let! x = 10;`
- ❌ `x := 10;` → ✅ `let x = 10;`
- ❌ `mut x` → ✅ `let! x`
- ❌ `&mut T` → ✅ `&T!`
- ✅ Keep `let x: i32 = 10;` (explicit type)
