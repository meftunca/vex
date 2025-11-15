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
**Status:** IMPLEMENTED ✅  
**Description:**  
The `vex format` command is implemented and uses the `vex_formatter` crate for code formatting.

### 3. Check Command Implementation
**Status:** IMPLEMENTED ✅  
**Description:**  
The `vex check` command performs syntax checking without full compilation.

## 🐛 Known Bugs

### 1. Emit SPIR-V Flag Not Implemented
**Status:** PARTIALLY IMPLEMENTED  
**Severity:** LOW  
**Priority:** MEDIUM  

**Description:**  
The `--emit-spirv` flag is defined in the compile command but not used in the implementation. The parameter is ignored with `_`.

**Location:**  
`vex-cli/src/main.rs`, line ~250: `emit_spirv: _,`

**Impact:**  
GPU SPIR-V emission is not available.

### 2. Error Handling Inconsistencies
**Status:** MINOR ISSUE  
**Severity:** LOW  
**Priority:** LOW  

**Description:**  
Some commands use `eprintln!` for errors and `std::process::exit(1)`, while others rely on `?` propagation. This creates inconsistent error reporting.

**Examples:**  
- Compile command: `eprintln!("❌ Dependency resolution failed: {}", e); std::process::exit(1);`  
- Other commands: Return `Result` with `?`  

**Impact:**  
Inconsistent user experience for error messages.

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

### 1. Interactive Mode
**Description:**  
Add an interactive REPL mode for quick testing and experimentation.

**Proposed Command:**  
```bash
vex repl
```

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

| Feature/Command | Priority | Effort | Impact | Status |
|----------------|----------|--------|--------|--------|
| Test Command   | CRITICAL | HIGH   | HIGH   | ✅ IMPLEMENTED |
| Format Command | HIGH     | MEDIUM | MEDIUM | ✅ IMPLEMENTED |
| Check Command  | HIGH     | LOW    | MEDIUM | ✅ IMPLEMENTED |
| Error Handling | MEDIUM   | LOW    | LOW    | PENDING |
| SPIR-V Support | MEDIUM   | HIGH   | LOW    | PENDING |
| Coverage Integration | LOW    | MEDIUM | MEDIUM | PENDING |
| Fuzz Testing Integration | LOW    | HIGH   | LOW    | PENDING |

## 🤝 Contributing

To contribute fixes for these issues:

1. Check `TODO.md` for current development priorities
2. Implement the missing functionality
3. Add appropriate tests
4. Update documentation
5. Run `scripts/update_docs.sh` to refresh project status

---

*This document should be updated whenever new issues are discovered or features are implemented.*</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/vex-cli/ISSUES.md