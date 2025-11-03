# Vex Example Programs

This directory contains example programs demonstrating various Vex language features.

## 🎯 Working Examples (Parser + Codegen Complete)

### Basic Programs

These compile and run successfully with the current compiler:

| Program           | Description         | Features                          | Output         |
| ----------------- | ------------------- | --------------------------------- | -------------- |
| **calculator.vx** | Basic arithmetic    | Functions, parameters, operations | 13, 7, 30, 3   |
| **sum_array.vx**  | Array iteration     | Arrays, for loops                 | 15             |
| **gcd.vx**        | Euclidean algorithm | While loops, modulo               | 6              |
| **fibonacci.vx**  | Recursive Fibonacci | Recursion                         | 55             |
| **factorial.vx**  | Recursive factorial | Recursion                         | 120            |
| **strings.vx**    | String handling     | String type, literals             | Hello messages |

### Compile and run:

```bash
vex compile examples/calculator.vx
./vex-builds/calculator
```

## 🚧 Advanced Examples (Syntax Complete, Codegen TODO)

These demonstrate language features that parse correctly but need additional compiler support:

### Import System

- **with_imports.vx** - Import syntax demonstration
  - Parser: ✅ Complete
  - Module Resolution: 🚧 In progress
  - Status: Needs `ModuleResolver` integration

### Error Handling

- **error_handling.vx** - Union types `(T | error)`
  - Parser: ✅ Union types in AST
  - Codegen: ❌ Needs tagged union implementation
  - Status: Needs match expressions

### Traits

- **trait_example.vx** - Reader/Writer traits
  - Parser: 🚧 Needs trait/impl parsing
  - Codegen: ❌ Needs vtable or monomorphization
  - Status: Foundation in AST, parser TODO

### Async/Await

- **async_example.vx** - Async I/O operations
  - Parser: 🚧 Needs async keyword handling
  - Codegen: ❌ Needs coroutine lowering
  - Status: Requires state machine generation

### Concurrency

- **concurrent_tasks.vx** - Go keyword and channels
  - Parser: 🚧 Needs go keyword parsing
  - Codegen: ❌ Needs runtime task spawning
  - Status: Requires async runtime

### GPU Computing

- **gpu_vector_add.vx** - Simple GPU kernel
- **gpu_matmul.vx** - Matrix multiplication
  - Parser: 🚧 Needs launch keyword parsing
  - Codegen: ❌ Needs CUDA/Metal backend
  - Status: Requires GPU runtime

## 📚 Standard Library Examples

These demonstrate std library usage (needs module resolution):

### Network Programming

```vex
import { http } from "std";

let resp = await http.get("http://example.com");
print(resp.body);
```

### File I/O

```vex
import { io } from "std";

let content = await io.read_to_string("data.txt");
print(content);
```

### Testing

```vex
import { testing } from "std";

fn test_add(t: &mut testing.TestContext) {
    t.assert_eq(2 + 2, 4, "math works");
}
```

## 🎓 Learning Path

**Start here if you're new to Vex:**

1. **calculator.vx** - Basic functions and arithmetic
2. **sum_array.vx** - Arrays and loops
3. **fibonacci.vx** - Recursion
4. **strings.vx** - String handling
5. **with_imports.vx** - Module system
6. **trait_example.vx** - Traits and generics
7. **async_example.vx** - Async I/O
8. **concurrent_tasks.vx** - Concurrency
9. **gpu_matmul.vx** - GPU acceleration

## 🔧 Compilation Status

| Feature        | Parser | Codegen | Runtime | Status            |
| -------------- | ------ | ------- | ------- | ----------------- |
| Functions      | ✅     | ✅      | N/A     | Working           |
| Variables      | ✅     | ✅      | N/A     | Working           |
| Arrays         | ✅     | ✅      | N/A     | Working           |
| Loops          | ✅     | ✅      | N/A     | Working           |
| Strings        | ✅     | ✅      | N/A     | Working           |
| Imports        | ✅     | 🚧      | N/A     | Parser done       |
| Traits         | 🚧     | ❌      | N/A     | Foundation ready  |
| Async/Await    | 🚧     | ❌      | ❌      | Needs runtime     |
| Go keyword     | ❌     | ❌      | ❌      | Needs runtime     |
| Launch keyword | ❌     | ❌      | ❌      | Needs GPU runtime |

## 💡 Notes

### String Support (NEW!)

Strings are now fully supported:

```vex
let message: string = "Hello, World!";
print(message);
```

Strings compile to `i8*` pointers and use global string constants.

### Import System (NEW!)

Import syntax is parsed:

```vex
import { io, net } from "std";
import "std::http";
```

Module resolution coming soon to load actual std library files.

### Trait System (Foundation)

Trait tokens added to lexer:

- `trait` keyword
- `impl` keyword

Parser and codegen implementation in progress.

## 🚀 Next Steps

To make advanced examples work:

1. **Module Resolution** - Load vex-libs/std/ modules
2. **Trait Parsing** - Parse trait/impl blocks
3. **Error Types** - Implement tagged unions
4. **Match Expressions** - Pattern matching for error handling
5. **Async Codegen** - State machine generation
6. **Vex Runtime** - Rust-based with io_uring

## 📝 Contributing

Want to add more examples? Follow these guidelines:

1. **Working examples** - Should compile and run with current compiler
2. **Advanced examples** - Can demonstrate future features but add comments explaining status
3. **Document requirements** - Clearly mark which compiler features are needed
4. **Keep it simple** - One feature per example when possible
5. **Add to this README** - Update the tables above

## 🎯 Goal

Make every example program compilable and runnable as compiler features are implemented!

Currently: **6/15 examples fully working** ✅

Target: **15/15 examples working** by end of Q1 2026
