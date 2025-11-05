# Filesystem Package

File and path operations for Vex programs.

## Functions

```vex
fn read_file(path: string): string
fn write_file(path: string, content: string): i32
fn exists(path: string): bool
```

## Usage

```vex
import { fs } from "std";

fn main(): i32 {
    let content = fs.read_file("input.txt");

    fs.write_file("output.txt", "Hello, Vex!");

    if fs.exists("config.txt") {
        // ...
    }

    return 0;
}
```

## C Runtime

Uses `vex-clibs/`:

- `vex_file.c` - File I/O operations
- `vex_path.c` - Path manipulation
- `vex_io.c` - Low-level I/O

## Status

⚠️ **Placeholder implementations** - C runtime integration pending.
