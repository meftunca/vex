# Vex CLI Usage Documentation

## Overview

The Vex CLI (`vex`) is the command-line interface for the Vex programming language. It provides tools for project management, dependency handling, compilation, execution, testing, and code formatting. This document provides detailed usage instructions for all available commands.

## Installation

To install the Vex CLI, build it from source using Cargo:

```bash
git clone https://github.com/meftunca/vex.git
cd vex
cargo build --release
```

The binary will be available at `target/release/vex`. You can add it to your PATH or use it directly.

## General Usage

```bash
vex [COMMAND] [OPTIONS]
```

Use `vex --help` to see all available commands and global options.

## Commands

### Project Management

#### `vex new <NAME> [OPTIONS]`

Create a new Vex project with the specified name.

**Options:**
- `--path <PATH>`: Project path (default: `./<name>`)

**Examples:**
```bash
vex new my-project
vex new my-app --path /path/to/projects/my-app
```

This creates a new directory with a basic project structure including `vex.json`, `src/main.vx`, and other necessary files.

#### `vex init [PATH]`

Initialize a Vex project in an existing directory by creating a `vex.json` manifest file.

**Arguments:**
- `PATH`: Project path (default: current directory)

**Examples:**
```bash
vex init
vex init /path/to/existing/project
```

### Dependency Management

#### `vex add <PACKAGE> [OPTIONS]`

Add a dependency to the project.

**Arguments:**
- `PACKAGE`: Package URL (e.g., `github.com/user/repo@v1.0.0`)

**Options:**
- `--version <VERSION>`: Version (if not specified in package URL)

**Examples:**
```bash
vex add github.com/user/math-lib@v1.2.0
vex add github.com/user/http-client --version ^2.0.0
```

#### `vex remove <PACKAGE>`

Remove a dependency from the project.

**Arguments:**
- `PACKAGE`: Package name

**Examples:**
```bash
vex remove github.com/user/math-lib
```

#### `vex list`

List all project dependencies.

**Examples:**
```bash
vex list
```

#### `vex update`

Update all dependencies to their latest compatible versions.

**Examples:**
```bash
vex update
```

#### `vex clean`

Clean cache and build artifacts.

**Examples:**
```bash
vex clean
```

### Compilation and Execution

#### `vex compile <INPUT> [OPTIONS]`

Compile a Vex source file to an executable.

**Arguments:**
- `INPUT`: Input `.vx` file

**Options:**
- `-o, --output <OUTPUT>`: Output file path
- `--simd`: Enable SIMD optimizations
- `--gpu`: Enable GPU support
- `-O, --opt-level <LEVEL>`: Optimization level (0-3, default: 2)
- `--emit-llvm`: Emit LLVM IR instead of executable
- `--emit-spirv`: Emit SPIR-V (for GPU functions)
- `--locked`: Use lock file (CI mode - fails if lock file is invalid)
- `--json`: Output diagnostics as JSON (for IDE integration)

**Examples:**
```bash
vex compile main.vx
vex compile src/app.vx -o my-app
vex compile --simd --gpu -O 3 main.vx
vex compile --emit-llvm main.vx
vex compile --locked --json main.vx
```

#### `vex run [INPUT] [OPTIONS] [ARGS...]`

Compile and execute a Vex source file, or execute code from a string.

**Arguments:**
- `INPUT`: Input `.vx` file or code string (when using `-c`)
- `ARGS`: Arguments to pass to the program

**Options:**
- `-c, --code <CODE>`: Execute code from string (like `node -c`)
- `--json`: Output diagnostics as JSON
- `-O, --opt-level <LEVEL>`: Optimization level (0-3, default: 0)

**Examples:**
```bash
vex run main.vx
vex run main.vx -- arg1 arg2
vex run -c "print('Hello, World!')"
vex run --json main.vx
```

### Code Quality

#### `vex check <INPUT>`

Check syntax of a Vex source file without compiling.

**Arguments:**
- `INPUT`: Input `.vx` file

**Examples:**
```bash
vex check main.vx
```

#### `vex format <INPUT> [OPTIONS]`

Format Vex source code.

**Arguments:**
- `INPUT`: Input `.vx` file

**Options:**
- `-i, --in-place`: Format the file in place

**Examples:**
```bash
vex format main.vx
vex format -i main.vx
```

### Testing

#### `vex test [PATTERN] [OPTIONS]`

Run tests in the project.

**Arguments:**
- `PATTERN`: Specific test file or pattern (default: all tests)

**Options:**
- `-v, --verbose`: Run tests verbosely
- `--no-parallel`: Disable parallel test execution
- `--timeout <SECONDS>`: Custom timeout in seconds
- `--bench`: Run benchmarks instead of tests
- `--benchtime <DURATION>`: Benchmark execution time (default: 1s)
- `--count <N>`: Number of benchmark iterations (default: 1)
- `--benchmem`: Show memory allocation statistics for benchmarks
- `--coverage`: Generate coverage report
- `--coverprofile <FILE>`: Coverage profile output file
- `--covermode <MODE>`: Coverage mode: set, count, or atomic (default: set)
- `--short`: Run in short mode (skip slow tests)
- `--fuzz <FUZZ_TARGET>`: Run fuzzing tests
- `--fuzztime <DURATION>`: Fuzzing execution time (default: 10s)
- `--run <REGEX>`: Filter tests by name (regex)

**Examples:**
```bash
vex test
vex test my_test
vex test --verbose
vex test --bench
vex test --coverage --coverprofile coverage.out
vex test --fuzz my_fuzz_target --fuzztime 30s
vex test --run ".*integration.*"
```

## Project Structure

A typical Vex project has the following structure:

```
my-project/
├── vex.json          # Project manifest
├── vex.lock          # Lock file (generated)
├── src/
│   ├── lib.vx        # Library entry point
│   ├── main.vx       # Executable entry point
│   └── mod.vx        # Module declarations
├── tests/            # Test files
├── examples/         # Example code
└── vex-builds/       # Build artifacts (generated)
```

## Configuration

### vex.json

The project manifest file contains metadata and configuration:

```json
{
  "name": "my-project",
  "version": "1.0.0",
  "description": "A Vex project",
  "authors": ["Your Name <you@example.com>"],
  "license": "MIT",
  "dependencies": {
    "github.com/user/math-lib": "v1.2.0",
    "github.com/user/http-client": "^2.0.0"
  },
  "targets": {
    "debug": {
      "opt-level": 0
    },
    "release": {
      "opt-level": 3
    }
  },
  "main": "src/main.vx",
  "bin": {
    "my-app": "src/main.vx",
    "cli-tool": "src/cli.vx"
  },
  "native": {
    "include": ["vendor/include"],
    "libs": ["ssl", "crypto"],
    "flags": ["-O3", "-march=native"]
  }
}
```

### Dependency Versioning

- `"v1.2.3"`: Exact version
- `"^1.2.0"`: Compatible with 1.x
- `"1.0.0..2.0.0"`: Range
- `"*"`: Latest

## Common Workflows

### Starting a New Project

```bash
vex new my-app
cd my-app
vex run
```

### Adding Dependencies

```bash
vex add github.com/vex-lang/std-http
vex add github.com/vex-lang/std-json@v2.0.0
```

### Building for Production

```bash
vex compile -O 3 --simd src/main.vx -o my-app
```

### CI/CD Build

```bash
vex compile --locked --json src/main.vx
```

### Formatting Code

```bash
vex format -i src/main.vx
# Or format all files
find src -name "*.vx" -exec vex format -i {} \;
```

### Running Tests

```bash
vex test
vex test --verbose
vex test --coverage
```

## Error Handling

The CLI provides detailed error messages. Use `--json` flag with compilation commands to get structured error output suitable for IDE integration.

## Version

Current version: 0.2.0

For more information about the Vex language, see the [main documentation](../docs/REFERENCE.md).</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/vex-cli/README.md