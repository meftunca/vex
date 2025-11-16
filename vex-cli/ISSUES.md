# Vex CLI Issues and Missing Features

This document tracks known issues, bugs, and missing features in the Vex CLI tool.

**Last Updated:** November 16, 2025

## 🚧 Missing Features

### 1. Test Command Implementation

**Status:** IMPLEMENTED ✅  
**Description:**  
The `vex test` command is fully implemented with support for:

- Test discovery with glob patterns
- Parallel execution
- Benchmarking
- Coverage reporting (framework exists, needs integration)
- Fuzz testing (framework exists, needs integration)
- Timeout handling
- Verbose output

**Note:** While the command is implemented, some advanced features like coverage and fuzzing may need additional tooling integration.

### 2. Format Command Implementation

**Status:** PARTIALLY IMPLEMENTED  
**Severity:** HIGH  
**Priority:** CRITICAL

**Description:**  
The `vex format` command exists but vex-formatter is only ~40% complete. Major gaps include:

**Missing AST Node Coverage:**

- All expression types (literals, operators, calls, indexing, ranges, async/await, match, struct literals, closures, type casting)
- Statement types (switch, for loops, pattern destructuring, compound assignments)
- Type system features (union types, conditional types, function types)
- Import formatting (currently just `// import`)
- Contract/trait features (associated types, default implementations)

**Configuration Not Used:**

- `max_width`, `brace_style`, `trailing_comma`, `quote_style` options are defined but ignored
- Rules modules (spacing, indentation, expressions) exist but not integrated

**Impact:**  
Code formatting is unreliable and incomplete. Many Vex language constructs are not formatted at all.

### 3. Check Command Implementation

**Status:** IMPLEMENTED ✅  
**Description:**  
The `vex check` command performs syntax checking without full compilation.

## 🐛 Known Bugs

### 1. Emit SPIR-V Flag Partially Implemented

**Status:** PARTIALLY IMPLEMENTED ✅  
**Severity:** LOW  
**Priority:** MEDIUM

**Description:**  
The `--emit-spirv` flag now generates LLVM IR with GPU annotations (`.spirv.ll` file) but doesn't emit final SPIR-V binary yet. Full SPIR-V binary emission requires LLVM SPIR-V backend integration.

**Current Functionality:**
- ✅ Flag is recognized and processed
- ✅ Requires `--gpu` flag to work
- ✅ Generates LLVM IR for SPIR-V target
- ⚠️ Final `.spv` binary emission not yet implemented

**Usage:**
```bash
vex compile --gpu --emit-spirv shader.vx
# Outputs: vex-builds/shader.spirv.ll
# Convert with: llvm-spirv shader.spirv.ll -o shader.spv
```

**Impact:**  
GPU SPIR-V development requires manual llvm-spirv conversion step.

### 2. Error Handling Inconsistencies

**Status:** IMPROVED ✅  
**Severity:** LOW  
**Priority:** LOW

**Description:**  
Most error handling has been standardized to use proper `Result` propagation with JSON support. Dependency resolution errors now return proper errors instead of using `std::process::exit(1)`.

**Improvements:**
- ✅ Dependency resolution uses proper error propagation
- ✅ JSON error output for dependency failures
- ✅ Consistent error formatting

**Remaining Minor Issues:**
- Some legacy code paths may still use different error styles

**Impact:**  
Mostly resolved - error handling is now consistent across most commands.

## 🔧 Technical Debt

### 1. Hardcoded Binary Path

**Status:** TECHNICAL DEBT  
**Severity:** LOW  
**Priority:** LOW

**Description:**  
The CLI assumes the binary is at `~/.cargo/target/debug/vex`, which may not be correct in all environments.

**Location:**  
Various scripts and documentation reference this path.

**Impact:**  
May not work in different installation setups.

### 2. Limited Error Diagnostics

**Status:** ENHANCEMENT NEEDED  
**Severity:** MEDIUM  
**Priority:** MEDIUM

**Description:**  
While JSON diagnostics are supported for compilation, other commands lack structured error output.

**Impact:**  
IDE integration is limited to compilation errors only.

## 📋 Feature Requests

### 1. Interactive Mode (REPL)

**Status:** IMPLEMENTED ✅  
**Description:**  
Interactive REPL mode for quick testing and experimentation is now available.

**Usage:**
```bash
vex repl                  # Start REPL
vex repl --load file.vx   # Load file into context
vex repl --verbose        # Verbose mode
```

**Features:**
- ✅ Interactive code execution
- ✅ Context management (clear, show)
- ✅ File loading
- ✅ Help system
- ⚠️ JIT execution not yet implemented (parses only)

**REPL Commands:**
- `exit/quit` - Exit REPL
- `help` - Show commands
- `clear` - Clear context
- `show` - Show current context
- `load <file>` - Load file

### 2. Build Configuration Profiles

**Description:**  
Support for different build profiles (debug, release, test) with predefined settings.

### 3. Cross-Platform Testing

**Description:**  
Built-in support for testing across multiple platforms and architectures.

### 4. Dependency Graph Visualization

**Description:**  
Command to visualize the dependency graph of the project.

**Proposed Command:**

```bash
vex deps --graph
```

## 🧪 Testing Gaps

### 1. CLI Integration Tests

**Status:** MISSING  
**Description:**  
No automated tests for CLI command functionality.

**Impact:**  
CLI changes may break existing functionality without detection.

### 2. Error Path Testing

**Status:** MISSING  
**Description:**  
Error conditions and edge cases are not tested.

## 📚 Documentation Issues

### 1. Command Examples

**Status:** INCOMPLETE  
**Description:**  
Some advanced command options lack examples in the documentation.

### 2. Troubleshooting Guide

**Status:** MISSING  
**Description:**  
No guide for common issues and their solutions.

## 🔄 Migration Notes

### From Development to Production

- Implement missing commands before v1.0 release
- Add comprehensive error handling
- Implement testing framework integration
- Add CI/CD support with proper exit codes

## 📊 Priority Matrix

| Feature/Command          | Priority | Effort | Impact | Status         |
| ------------------------ | -------- | ------ | ------ | -------------- |
| Test Command             | CRITICAL | HIGH   | HIGH   | ✅ IMPLEMENTED |
| Format Command           | HIGH     | MEDIUM | MEDIUM | ✅ IMPLEMENTED |
| Check Command            | HIGH     | LOW    | MEDIUM | ✅ IMPLEMENTED |
| Error Handling           | MEDIUM   | LOW    | LOW    | PENDING        |
| SPIR-V Support           | MEDIUM   | HIGH   | LOW    | PENDING        |
| Coverage Integration     | LOW      | MEDIUM | MEDIUM | PENDING        |
| Fuzz Testing Integration | LOW      | HIGH   | LOW    | PENDING        |

## 🤝 Contributing

To contribute fixes for these issues:

1. Check `TODO.md` for current development priorities
2. Implement the missing functionality
3. Add appropriate tests
4. Update documentation
5. Run `scripts/update_docs.sh` to refresh project status

---

_This document should be updated whenever new issues are discovered or features are implemented._</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/vex-cli/ISSUES.md
