// Core builtin functions: print, println, panic, assert, unreachable
//
// ROADMAP: Print Function Unification (Future: vex-libs/std/io)
// ============================================================
//
// Current State:
//   - print(...args)   → Go-style variadic (space-separated, no newline)
//   - println(...args) → Go-style variadic + newline
//
// TODO (Phase 1): Format String Support
//   - Detect: If first arg is string literal with '{}' → format mode
//   - Modes:
//     1. print("x = {}, y = {}", 42, 3.14)  → vex_print_fmt()
//     2. print("x =", 42, "y =", 3.14)      → vex_print_args()
//   - Placeholders:
//     - {}     → Default format
//     - {:?}   → Debug format
//     - {:.N}  → Float precision
//     - {:x}   → Hex format
//   - Implementation: Add format string parsing to detect_print_mode()
//
// TODO (Phase 2): Move to Stdlib
//   - Move print/println to vex-libs/std/io.vx
//   - Keep only low-level C FFI in builtins (vex_print_args, vex_print_fmt)
//   - Example stdlib implementation:
//     ```vex
//     pub fn println(...args) {
//         print(...args);
//         print("\n");
//     }
//     ```
//
// C Runtime Functions (already implemented in vex_io.c):
//   - vex_print_args(count, VexValue*)        → Go-style (current)
//   - vex_println_args(count, VexValue*)      → Go-style + newline (current)
//   - vex_print_fmt(fmt, count, VexValue*)    → Rust-style format (TODO: expose)
//   - vex_println_fmt(fmt, count, VexValue*)  → Rust-style format + newline (TODO: expose)

mod assertions;
mod print_execution;
mod print_formatting;

pub use assertions::{
    builtin_assert, builtin_panic, builtin_print, builtin_println, builtin_unreachable,
};
pub use print_formatting::{compile_print_call, compile_typesafe_format};
