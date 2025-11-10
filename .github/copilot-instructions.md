# Vex Language Compiler - AI Agent Instructions

**Project:** Vex - Modern systems programming language
**Version:** 0.1.2 (Syntax v0.1.2)
**Last Updated:** November 9, 2025

## 🎯 Core Principles

1. **Check reference documentation first** - See docs/REFERENCE.md, docs/PROJECT_STATUS.md for current specs
2. **No shortcuts** - Implement features properly, not quick hacks
3. **Comprehensive testing** - Test all edge cases, not just happy paths
4. **Parallel development** - If feature A needs feature B enhancement, develop both
5. **⚠️ ABSOLUTE SILENCE RULE** - **DO NOT** engage in conversation, explanations, or discussions unless explicitly asked. Work completely silently. Only provide minimal status updates at the very end.
6. **Minimal status format** - Final report MUST be: `✅ [Task] → [Result] ([files changed])` - Nothing more.
7. **Use absolute paths** - Binary is at `~/.cargo/target/debug/vex`
8. **Follow Vex syntax v0.1.2** - Not Rust syntax (no `mut`, `->`, `::`)
9. **⚠️ CRITICAL: NO `::` operator!** - Use `. ` for all member access (`Vec.new()` not `Vec::new()`, `Some(x)` not `Option::Some(x)`)
10. **⚠️ FILE SIZE LIMIT: 400 LINES MAX** - **MANDATORY** Rust files MUST NOT exceed 400 lines. Split logically into modules when approaching this limit.

## 📚 Reference Documentation

**For detailed information, always check:**

- `docs/REFERENCE.md` - Complete language syntax and API reference
- `docs/PROJECT_STATUS.md` - Current test status, line counts, feature status
- `docs/ARCHITECTURE.md` - Detailed implementation architecture
- `TODO.md` - Current development priorities
- `Specifications/` - Formal language specifications

**These files are automatically updated by `scripts/update_docs.sh`**

## 🚀 Quick Start

```bash
# Build
cargo build

# Run file
~/.cargo/target/debug/vex run examples/hello.vx

# Run tests
./test_all.sh

# Update documentation
./scripts/update_docs.sh
```

---

*This file contains only immutable core rules. All project details are in reference documentation.*
