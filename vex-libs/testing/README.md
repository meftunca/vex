# Testing Package

Test assertions and utilities for Vex programs.

## Functions

```vex
fn assert(condition: bool)
fn assert_eq<T>(left: T, right: T)
fn assert_ne<T>(left: T, right: T)
```

## Usage

```vex
import { testing } from "std";

fn test_addition() {
    testing.assert_eq(2 + 2, 4);
    testing.assert_ne(2 + 2, 5);
    testing.assert(true);
}

fn main(): i32 {
    test_addition();
    return 0;
}
```

## C Runtime

Uses `vex-clibs/vex_testing.c`:

- `vex_test_start()`
- `vex_test_pass()`
- `vex_test_fail()`
- `vex_test_summary()`

## Future Features

- `#[test]` attribute for auto-registration
- Test runner with colored output
- Benchmark support

## Status

⚠️ **Placeholder implementations** - C runtime integration pending.
