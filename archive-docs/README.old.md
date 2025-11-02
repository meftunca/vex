# Vex Programming Language

**Version:** 0.2.0

Vex is a modern, statically-typed, compiled systems programming language designed for high-performance computing on modern hardware (multi-core CPUs, SIMD, GPU, fast I/O).

## Philosophy

Vex combines:

- The simplicity of Go
- Memory safety guarantees of Rust (with a simpler model)
- The flexible type system of TypeScript (e.g., Union Types)
- Native support for SIMD and GPU computing

## Features

- 🚀 **High Performance**: LLVM-based compilation with SIMD auto-vectorization
- 🎮 **GPU Computing**: First-class GPU support via SPIR-V (Vulkan/OpenCL/WebGPU)
- ⚡ **Async I/O**: Built on io_uring for efficient concurrent operations
- 🔒 **Memory Safety**: Simple reference model without complex lifetimes
- 🎯 **Modern Syntax**: Clean, expressive syntax inspired by Go and TypeScript

## Project Structure

```
vex_lang/
├── vex-lexer/      # Tokenization (logos)
├── vex-parser/     # Grammar and AST generation (lalrpop)
├── vex-ast/        # AST node definitions and type system
├── vex-compiler/   # Code generation (LLVM IR & SPIR-V)
├── vex-runtime/    # Async runtime (tokio + io_uring)
├── vex-cli/        # Command-line interface
└── examples/       # Example .vx programs
```

## Getting Started

### Prerequisites

- Rust 1.70 or later
- LLVM 16.0
- (Optional) Vulkan SDK for GPU support

### Building

```bash
cargo build --release
```

### Running

```bash
./target/release/vex compile examples/hello.vx -o hello
./hello
```

## Language Overview

### Hello, Vex!

```javascript
import { io, log } from "std";

fn main(): error {
    log.info("Vex v0.2 running.");
    io.print(f"1 + 2 = {1 + 2}\n");
    return nil;
}
```

### SIMD Auto-vectorization

```javascript
fn add_vectors(a: &[f32; 4], b: &[f32; 4], out: &mut [f32; 4]) {
    @vectorize
    for i in 0..a.len() {
        out[i] = a[i] + b[i];
    }
}
```

### GPU Computing

```javascript
gpu fn matrix_multiply(a: &[f32], b: &[f32], out: &mut [f32], size: u32) {
    let x = @gpu.global_id.x;
    let y = @gpu.global_id.y;

    if x >= size || y >= size {
        return;
    }

    let sum: f32 = 0.0;
    for k in 0..size {
        sum += a[y * size + k] * b[k * size + x];
    }
    out[y * size + x] = sum;
}

fn main(): error {
    let N: u32 = 1024;
    // ... initialize data ...

    await launch matrix_multiply[N, N](a_data, b_data, &mut out_data, N);
    return nil;
}
```

## Documentation

See [Specification.md](Specification.md) for the complete language specification.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

## Contributing

Contributions are welcome! Please read our contributing guidelines first.
