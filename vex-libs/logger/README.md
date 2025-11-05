# Logger Package

Simple logging utilities for Vex programs.

## Functions

```vex
fn debug(msg: string)  // Debug message
fn info(msg: string)   // Info message
fn warn(msg: string)   // Warning message
fn error(msg: string)  // Error message
```

## Usage

```vex
import { logger } from "std";

fn main(): i32 {
    logger.info("Application started");
    logger.debug("Debug info");
    logger.warn("Warning");
    logger.error("Error occurred");
    return 0;
}
```

## C Runtime

Uses `vex-clibs/vex_io.c`:

- `vex_print()`
- `vex_println()`

## Zero-Cost

Compiles to direct `write(2)` syscalls in optimized builds.
