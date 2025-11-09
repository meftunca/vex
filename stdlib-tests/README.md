# Vex Standard Library Test Suite

Comprehensive tests for all stdlib modules.

## Test Status

| Module          | Status         | Native FFI | Notes                         |
| --------------- | -------------- | ---------- | ----------------------------- |
| **io**          | ✅ PASSING     | Yes        | println via vex_runtime       |
| **math**        | ✅ PASSING     | Yes        | LLVM intrinsics + stdlib      |
| **core**        | ✅ PASSING     | No         | Language built-ins            |
| **fs**          | ✅ PASSING     | Yes        | file_exists FFI               |
| **testing**     | ✅ PASSING     | No         | Pure Vex assertions           |
| **time**        | ✅ PASSING     | Yes        | libvextime.a (SIMD optimized) |
| **collections** | ⚠️ PLACEHOLDER | No         | Not yet implemented           |
| **string**      | ⚠️ PLACEHOLDER | No         | Not yet implemented           |
| **env**         | 📝 PENDING     | -          | Module not integrated         |
| **process**     | 📝 PENDING     | -          | Module not integrated         |
| **path**        | 📝 PENDING     | -          | Module not integrated         |
| **crypto**      | 📝 PENDING     | -          | Module not integrated         |
| **encoding**    | 📝 PENDING     | -          | Module not integrated         |
| **net**         | 📝 PENDING     | -          | Module not integrated         |
| **http**        | 📝 PENDING     | -          | Module not integrated         |
| **db**          | 📝 PENDING     | -          | Module not integrated         |

**Total:** 6/8 PASSING, 2 PLACEHOLDER, 10 PENDING

## Running Tests

```bash
# Run all tests (must be from workspace root)
cd /Users/mapletechnologies/Desktop/big_projects/vex_lang
for f in stdlib-tests/test_*.vx; do
  echo "=== $(basename $f) ==="
  ~/.cargo/target/debug/vex run "$f"
done

# Run individual test
~/.cargo/target/debug/vex run stdlib-tests/test_io.vx
~/.cargo/target/debug/vex run stdlib-tests/test_math.vx
~/.cargo/target/debug/vex run stdlib-tests/test_time.vx
```

## Native Library Integration

### Time Module (✅ COMPLETE)

- **C Runtime:** `vex-runtime/c/vex_time/`
- **Static Library:** `libvextime.a` (pre-compiled, SIMD optimized)
- **FFI Functions:** `vt_monotonic_now_ns`, `vt_sleep_ns`
- **Configuration:** `vex-libs/std/time/vex.json`

Native dependencies are automatically resolved via `vex.json`:

```json
{
  "native": {
    "libraries": ["../../../vex-runtime/c/vex_time/libvextime.a"],
    "include_dirs": ["../../../vex-runtime/c/vex_time/include"]
  }
}
```

**How it works:**

1. Import resolution checks each module's `vex.json`
2. If native config exists, NativeLinker processes it
3. Library paths are resolved to absolute paths
4. Arguments are collected in `ModuleResolver.native_linker_args`
5. Arguments are added to clang command during linking

## Test Organization

Each test file:

1. Tests module imports
2. Tests basic functionality
3. Returns 0 on success, 1 on failure
4. Provides clear output

## Integration Status

### ✅ Phase 1: Import System (COMPLETE)

- Borrow checker import fix
- Const import support
- Function import support

### 🔄 Phase 2: Module Integration (IN PROGRESS)

- Math module: ✅ COMPLETE
- IO module: ✅ COMPLETE
- FS module: ⚠️ FFI only
- Other modules: 📝 Pending

### 📋 Phase 3: Full Stdlib (PLANNED)

- All modules with full import support
- Comprehensive test coverage
- Documentation
