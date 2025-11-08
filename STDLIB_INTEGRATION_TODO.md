# Vex Standard Library - Implementation Roadmap

**Status:** Design Complete ✅ | Implementation Pending 🚧

**Last Updated:** November 8, 2025

---

## 🎯 PHASE 0: Foundation & Fixes (Week 1 - IMMEDIATE)

**Priority:** 🔴 CRITICAL - Required before any stdlib integration

### Task 0.1: C Runtime String ABI Fix ⚠️ BREAKING CHANGE

**Problem:** Current C runtime uses null-terminated strings, but Vex uses fat pointers `{ ptr, len }`.

**Current (WRONG):**

```c
// vex-runtime/c/vex.h
void vex_print(const char *s);      // ❌ C-string
void vex_println(const char *s);    // ❌ No length
```

**Target (CORRECT):**

```c
// vex-runtime/c/vex.h
void vex_print(const char *ptr, uint64_t len);
void vex_println(const char *ptr, uint64_t len);
void vex_eprint(const char *ptr, uint64_t len);
void vex_eprintln(const char *ptr, uint64_t len);
```

**Files to Update:**

- [ ] `vex-runtime/c/vex.h` - Update function signatures
- [ ] `vex-runtime/c/vex_io.c` - Update implementations
- [ ] `vex-runtime/c/vex_string.c` - Update string functions
- [ ] `vex-libs/std/io/src/lib.vx` - Update extern declarations
- [ ] All existing Vex code using `print()`/`println()`

**Estimate:** 4-6 hours

**Test:**

```vex
fn main() {
    print("Hello");           // Should pass ptr + len
    println("World");         // Should pass ptr + len
    let s = "Test";
    println(s);               // Should work with string variable
}
```

---

### Task 0.2: Platform Detection Implementation

**Goal:** Implement compile-time platform/arch detection for stdlib file selection.

**Location:** `vex-compiler/src/resolver/platform.rs` (NEW)

**Implementation:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    BSD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X64,
    Arm64,
    Arm32,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
        return Platform::BSD;

        #[cfg(not(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "windows",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd"
        )))]
        compile_error!("Unsupported platform");
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Linux => "linux",
            Platform::MacOS => "macos",
            Platform::Windows => "windows",
            Platform::BSD => "bsd",
        }
    }
}

impl Arch {
    pub fn current() -> Self {
        #[cfg(target_arch = "x86_64")]
        return Arch::X64;

        #[cfg(target_arch = "aarch64")]
        return Arch::Arm64;

        #[cfg(target_arch = "arm")]
        return Arch::Arm32;

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm")))]
        compile_error!("Unsupported architecture");
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
            Arch::Arm32 => "arm32",
        }
    }
}
```

**Files to Create:**

- [ ] `vex-compiler/src/resolver/platform.rs`

**Estimate:** 2 hours

---

### Task 0.3: Stdlib Syntax Validation

**Goal:** Verify all 17 stdlib packages use correct Vex v0.9 syntax.

**Check List:**

- [ ] No `->` (use `:` for return types)
- [ ] No `::` (use `.` for namespaces)
- [ ] No `mut` (use `!` suffix for mutable)
- [ ] `extern "C"` blocks correct
- [ ] All types match C runtime

**Script:**

```bash
#!/bin/bash
# scripts/validate_stdlib_syntax.sh

for module in vex-libs/std/*/src/*.vx; do
    echo "Checking $module..."

    # Check for deprecated syntax
    grep -n "\->" "$module" && echo "❌ Found -> in $module"
    grep -n "::" "$module" && echo "❌ Found :: in $module"
    grep -n "mut " "$module" && echo "❌ Found mut in $module"

    # Verify extern "C" blocks
    grep -A5 'extern "C"' "$module"
done
```

**Files to Update:** All `vex-libs/std/*/src/*.vx` files

**Estimate:** 4-6 hours

---

## 🏗️ PHASE 1: Compiler Integration (Week 2-3)

**Priority:** 🔴 HIGH - Core infrastructure

### Task 1.1: StdlibResolver Implementation

**Goal:** Resolve `import { foo } from "module"` to `vex-libs/std/module/src/lib.vx`.

**Location:** `vex-compiler/src/resolver/stdlib_resolver.rs` (NEW)

**Features:**

- Module name → file path resolution
- Platform-specific file selection (priority chain)
- Built-in module detection (no vex.json lookup)
- Error handling (ModuleNotFound)

**Implementation:**

```rust
pub struct StdlibResolver {
    stdlib_root: PathBuf,
    platform: Platform,
    arch: Arch,
}

impl StdlibResolver {
    pub fn new() -> Self {
        Self {
            stdlib_root: PathBuf::from("vex-libs/std"),
            platform: Platform::current(),
            arch: Arch::current(),
        }
    }

    pub fn resolve_module(&self, module_name: &str) -> Result<PathBuf, ResolveError> {
        // Priority chain:
        // 1. lib.{os}.{arch}.vx (e.g., lib.linux.x64.vx)
        // 2. lib.{arch}.vx      (e.g., lib.x64.vx)
        // 3. lib.{os}.vx        (e.g., lib.linux.vx)
        // 4. lib.vx             (fallback)

        let module_dir = self.stdlib_root.join(module_name);
        let candidates = vec![
            module_dir.join(format!("src/lib.{}.{}.vx", self.platform.as_str(), self.arch.as_str())),
            module_dir.join(format!("src/lib.{}.vx", self.arch.as_str())),
            module_dir.join(format!("src/lib.{}.vx", self.platform.as_str())),
            module_dir.join("src/lib.vx"),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        Err(ResolveError::ModuleNotFound(module_name.to_string()))
    }

    pub fn is_stdlib_module(&self, module_name: &str) -> bool {
        const STDLIB_MODULES: &[&str] = &[
            "io", "core", "collections", "string", "memory", "sync",
            "time", "net", "encoding", "crypto", "db", "strconv",
            "path", "http", "json", "fmt", "testing"
        ];
        STDLIB_MODULES.contains(&module_name)
    }
}
```

**Files to Create:**

- [ ] `vex-compiler/src/resolver/mod.rs` (update)
- [ ] `vex-compiler/src/resolver/stdlib_resolver.rs`

**Integration:**

- [ ] Update `vex-compiler/src/module_resolver.rs` to use StdlibResolver
- [ ] Add tests in `vex-compiler/tests/stdlib_resolver_tests.rs`

**Estimate:** 1 day (8 hours)

**Test:**

```rust
#[test]
fn test_resolve_io_module() {
    let resolver = StdlibResolver::new();
    let path = resolver.resolve_module("io").unwrap();
    assert!(path.ends_with("vex-libs/std/io/src/lib.vx"));
}

#[test]
fn test_platform_specific_resolution() {
    let resolver = StdlibResolver::new();
    let path = resolver.resolve_module("net").unwrap();

    #[cfg(target_os = "linux")]
    assert!(path.to_str().unwrap().contains("lib.linux.vx"));

    #[cfg(target_os = "macos")]
    assert!(path.to_str().unwrap().contains("lib.macos.vx"));
}
```

---

### Task 1.2: FFI Bridge Implementation

**Goal:** Convert `extern "C"` declarations to LLVM IR without libclang.

**Location:** `vex-compiler/src/codegen_ast/ffi_bridge.rs` (NEW)

**Features:**

- Parse `extern "C"` blocks from Vex AST
- Vex type → LLVM type mapping
- Generate LLVM function declarations
- Set C ABI calling convention

**Implementation:**

```rust
pub struct FFIBridge<'ctx> {
    context: &'ctx Context,
    module: &'ctx Module<'ctx>,
}

impl<'ctx> FFIBridge<'ctx> {
    pub fn new(context: &'ctx Context, module: &'ctx Module<'ctx>) -> Self {
        Self { context, module }
    }

    pub fn generate_extern_declaration(
        &self,
        func: &ExternFunction,
    ) -> FunctionValue<'ctx> {
        // 1. Map parameter types
        let param_types: Vec<BasicMetadataTypeEnum> = func.params
            .iter()
            .map(|p| self.vex_type_to_llvm(&p.ty))
            .collect();

        // 2. Map return type
        let return_type = if let Some(ret) = &func.return_type {
            self.vex_type_to_llvm(ret)
        } else {
            self.context.void_type().into()
        };

        // 3. Create function type
        let fn_type = return_type.fn_type(&param_types, false);

        // 4. Add to module
        let fn_value = self.module.add_function(&func.name, fn_type, None);

        // 5. Set C calling convention
        fn_value.set_call_conventions(llvm_sys::LLVMCallConv::LLVMCCallConv as u32);

        fn_value
    }

    fn vex_type_to_llvm(&self, ty: &Type) -> BasicMetadataTypeEnum<'ctx> {
        match ty {
            Type::I8 => self.context.i8_type().into(),
            Type::I16 => self.context.i16_type().into(),
            Type::I32 => self.context.i32_type().into(),
            Type::I64 => self.context.i64_type().into(),
            Type::U8 => self.context.i8_type().into(),
            Type::U16 => self.context.i16_type().into(),
            Type::U32 => self.context.i32_type().into(),
            Type::U64 => self.context.i64_type().into(),
            Type::F32 => self.context.f32_type().into(),
            Type::F64 => self.context.f64_type().into(),
            Type::Bool => self.context.bool_type().into(),

            // Pointers: *const T → T*
            Type::Pointer(inner, is_mutable) => {
                let inner_type = self.vex_type_to_llvm(inner);
                inner_type.ptr_type(AddressSpace::default()).into()
            },

            // String: { i8*, i64 } (fat pointer)
            Type::String => {
                let i8_ptr = self.context.i8_type().ptr_type(AddressSpace::default());
                let i64 = self.context.i64_type();
                self.context.struct_type(&[i8_ptr.into(), i64.into()], false).into()
            },

            _ => panic!("Unsupported FFI type: {:?}", ty),
        }
    }
}
```

**Files to Create:**

- [ ] `vex-compiler/src/codegen_ast/ffi_bridge.rs`
- [ ] `vex-compiler/tests/ffi_bridge_tests.rs`

**Integration:**

- [ ] Update `vex-compiler/src/codegen_ast/mod.rs` to use FFIBridge
- [ ] Call `generate_extern_declaration()` when compiling `extern "C"` blocks

**Estimate:** 2 days (16 hours)

**Test:**

```vex
// test_ffi.vx
extern "C" {
    fn test_add(a: i32, b: i32): i32;
    fn test_print(ptr: *const u8, len: u64);
}

fn main() {
    let result = test_add(10, 20);
    println(result);
}
```

Expected LLVM IR:

```llvm
declare i32 @test_add(i32, i32)
declare void @test_print(i8*, i64)
```

---

### Task 1.3: Inline Optimizer Implementation

**Goal:** Ensure stdlib wrappers are completely inlined (zero-cost abstraction).

**Location:** `vex-compiler/src/codegen_ast/inline_optimizer.rs` (NEW)

**Features:**

- Detect `#[inline(always)]` attribute
- Set LLVM `alwaysinline` attribute
- Verify zero-cost (no unexpected calls)
- Post-optimization validation

**Implementation:**

```rust
pub struct InlineOptimizer<'ctx> {
    module: &'ctx Module<'ctx>,
}

impl<'ctx> InlineOptimizer<'ctx> {
    pub fn optimize_stdlib_wrappers(&self) {
        for function in self.module.get_functions() {
            if self.should_inline(&function) {
                self.set_always_inline(&function);
            }
        }
    }

    fn should_inline(&self, function: &FunctionValue<'ctx>) -> bool {
        // Check for #[inline(always)] in metadata
        // Or check function name pattern (stdlib wrappers)
        let name = function.get_name().to_str().unwrap();
        name.starts_with("std_") || name.starts_with("io_")
    }

    fn set_always_inline(&self, function: &FunctionValue<'ctx>) {
        let attr = self.module.get_context().create_enum_attribute(
            Attribute::get_named_enum_kind_id("alwaysinline"),
            0,
        );
        function.add_attribute(AttributeLoc::Function, attr);
    }

    pub fn verify_zero_cost(&self, function: &FunctionValue<'ctx>) -> Result<(), InlineError> {
        for bb in function.get_basic_blocks() {
            for instr in bb.get_instructions() {
                if let InstructionOpcode::Call = instr.get_opcode() {
                    let callee = instr.get_called_fn_value().unwrap();
                    let name = callee.get_name().to_str().unwrap();

                    // Only allow direct C calls (vex_*, vt_*)
                    if !name.starts_with("vex_") && !name.starts_with("vt_") {
                        return Err(InlineError::UnexpectedCall {
                            caller: function.get_name().to_str().unwrap().to_string(),
                            callee: name.to_string(),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}
```

**Files to Create:**

- [ ] `vex-compiler/src/codegen_ast/inline_optimizer.rs`
- [ ] `vex-compiler/tests/inline_tests.rs`

**Estimate:** 1 day (8 hours)

**Verification:**

```bash
# scripts/verify_inline.sh
vexc --emit-llvm -O3 test.vx -o test.ll
grep "call.*io_print" test.ll && echo "❌ NOT INLINED!" || echo "✅ INLINED"
```

---

## ⚡ PHASE 2: First Module Integration (Week 4)

**Priority:** 🟡 MEDIUM - Proof of concept

### Task 2.1: Implement `io` Module

**Goal:** Get `print()` and `println()` working end-to-end.

**Steps:**

1. Update C runtime signatures (Task 0.1)
2. Update `vex-libs/std/io/src/lib.vx`
3. Compile with StdlibResolver
4. Verify zero-cost with inline optimizer
5. Run benchmarks

**Test:**

```vex
import { print, println } from "io";

fn main() {
    print("Hello, ");
    println("World!");
}
```

**Expected:**

- Compile without errors
- Direct call to `vex_print()` / `vex_println()` in LLVM IR
- Performance within 5% of C baseline

**Estimate:** 2 days (16 hours)

---

### Task 2.2: Implement `string` Module

**Goal:** UTF-8 validation and basic string operations.

**Features:**

- `string.len()` → length in bytes
- `string.chars()` → UTF-8 char count
- `string.is_valid_utf8()` → SIMD validation

**C Runtime Functions:**

```c
uint64_t vex_utf8_len(const char* ptr, uint64_t byte_len);
bool vex_utf8_validate(const char* ptr, uint64_t len);
```

**Estimate:** 2 days (16 hours)

---

### Task 2.3: Implement `collections` Module

**Goal:** Vec<T> and HashMap<K,V> with zero-cost abstractions.

**Features:**

- `Vec.new()` → heap allocation
- `Vec.push()` → append with reallocation
- `HashMap.new()` → SwissTable (34M ops/s)

**C Runtime:**

```c
void* vex_vec_new(uint64_t elem_size, uint64_t capacity);
void vex_vec_push(void* vec, const void* elem);
void* vex_hashmap_new(uint64_t capacity);
```

**Estimate:** 3 days (24 hours)

---

## 🔗 PHASE 3: LTO Pipeline (Week 5)

**Priority:** 🟡 MEDIUM - Performance optimization

### Task 3.1: LLVM Bitcode Emission

**Goal:** Emit LLVM bitcode instead of object files for LTO.

**Location:** `vex-cli/src/main.rs`

**Implementation:**

```rust
fn compile_to_bitcode(source: &Path) -> Result<PathBuf> {
    let output = source.with_extension("bc");

    // Compile Vex → LLVM BC
    let context = Context::create();
    let mut codegen = ASTCodeGen::new(&context, "main");

    // ... parse and compile ...

    // Write bitcode
    codegen.module.write_bitcode_to_path(&output);
    Ok(output)
}
```

**Estimate:** 1 day (8 hours)

---

### Task 3.2: C Runtime Bitcode Compilation

**Goal:** Compile C runtime to LLVM bitcode for linking.

**Script:**

```bash
#!/bin/bash
# vex-runtime/c/compile_to_bc.sh

clang -emit-llvm -O3 -c vex_io.c -o vex_io.bc
clang -emit-llvm -O3 -c vex_string.c -o vex_string.bc
clang -emit-llvm -O3 -c vex_alloc.c -o vex_alloc.bc
# ... all runtime files ...

llvm-link vex_io.bc vex_string.bc vex_alloc.bc -o vex_runtime.bc
```

**Estimate:** 4 hours

---

### Task 3.3: LTO Integration

**Goal:** Link Vex bitcode + C runtime bitcode + optimize.

**Implementation:**

```bash
#!/bin/bash
# Link all bitcode
llvm-link main.bc vex_runtime.bc -o combined.bc

# Optimize
opt -O3 \
    -inline-threshold=2000 \
    -sroa -mem2reg -instcombine -simplifycfg \
    -loop-unroll -vectorize -slp-vectorize \
    combined.bc -o optimized.bc

# Generate native code
llc -O3 -march=native optimized.bc -o output.s
as output.s -o output.o
ld output.o -o final_binary
```

**Estimate:** 2 days (16 hours)

---

## 📊 PHASE 4: Testing & Validation (Week 6)

**Priority:** 🟢 LOW - Quality assurance

### Task 4.1: Performance Benchmarks

**Goal:** Verify zero-cost abstraction with real benchmarks.

**Benchmarks:**

- `println` loop (1M iterations) - target: <5% overhead vs C
- UTF-8 validation (10MB) - target: match SIMD C implementation
- HashMap insert (1M ops) - target: 30M+ ops/s

**Script:**

```bash
#!/bin/bash
# benchmarks/run_all.sh

echo "=== println benchmark ==="
time ./vex_println_bench
time ./c_println_bench

echo "=== UTF-8 validation ==="
time ./vex_utf8_bench
time ./c_utf8_bench

echo "=== HashMap benchmark ==="
time ./vex_hashmap_bench
time ./c_hashmap_bench
```

**Estimate:** 2 days (16 hours)

---

### Task 4.2: Assembly Inspection Automation

**Goal:** Automated verification that stdlib wrappers are inlined.

**Script:**

```bash
#!/bin/bash
# scripts/verify_zero_cost.sh

FUNCTIONS=("io_print" "io_println" "string_len" "vec_push")

vexc --emit-asm -O3 test.vx -o test.s

for func in "${FUNCTIONS[@]}"; do
    if grep "call.*$func" test.s; then
        echo "❌ $func NOT INLINED"
        exit 1
    fi
done

echo "✅ All stdlib functions inlined!"
```

**Estimate:** 1 day (8 hours)

---

### Task 4.3: Integration Tests

**Goal:** Test all 17 stdlib modules end-to-end.

**Test Structure:**

```
vex-libs/std/
├── io/tests/io_test.vx
├── string/tests/utf8_test.vx
├── collections/tests/vec_test.vx
└── ... (15 more modules)
```

**Test Runner:**

```bash
#!/bin/bash
# scripts/test_stdlib.sh

for module in vex-libs/std/*/tests/*.vx; do
    echo "Testing $module..."
    vexc "$module" -o test_bin
    ./test_bin || exit 1
done

echo "✅ All stdlib tests passed!"
```

**Estimate:** 3 days (24 hours)

---

## 📅 TIMELINE SUMMARY

| Phase                      | Duration | Tasks                                                | Priority    |
| -------------------------- | -------- | ---------------------------------------------------- | ----------- |
| **Phase 0: Foundation**    | Week 1   | C runtime fix, platform detection, syntax validation | 🔴 CRITICAL |
| **Phase 1: Compiler**      | Week 2-3 | StdlibResolver, FFI Bridge, Inline Optimizer         | 🔴 HIGH     |
| **Phase 2: First Modules** | Week 4   | io, string, collections                              | 🟡 MEDIUM   |
| **Phase 3: LTO**           | Week 5   | Bitcode emission, linking, optimization              | 🟡 MEDIUM   |
| **Phase 4: Testing**       | Week 6   | Benchmarks, assembly inspection, integration tests   | 🟢 LOW      |

**Total Estimate:** 6 weeks (~200 hours)

---

## ✅ CHECKLIST

### Week 1: Foundation

- [ ] Task 0.1: C runtime string ABI fix (6h)
- [ ] Task 0.2: Platform detection (2h)
- [ ] Task 0.3: Stdlib syntax validation (6h)

### Week 2-3: Compiler Integration

- [ ] Task 1.1: StdlibResolver (8h)
- [ ] Task 1.2: FFI Bridge (16h)
- [ ] Task 1.3: Inline Optimizer (8h)

### Week 4: First Modules

- [ ] Task 2.1: io module (16h)
- [ ] Task 2.2: string module (16h)
- [ ] Task 2.3: collections module (24h)

### Week 5: LTO Pipeline

- [ ] Task 3.1: Bitcode emission (8h)
- [ ] Task 3.2: C runtime bitcode (4h)
- [ ] Task 3.3: LTO integration (16h)

### Week 6: Testing

- [ ] Task 4.1: Benchmarks (16h)
- [ ] Task 4.2: Assembly inspection (8h)
- [ ] Task 4.3: Integration tests (24h)

---

## 🚀 GETTING STARTED

**Immediate Actions (TODAY):**

1. **Review C Runtime:**

   ```bash
   cd vex-runtime/c/
   grep -n "vex_print" vex.h vex_io.c
   # Verify current signatures
   ```

2. **Test Current State:**

   ```bash
   cd vex-libs/std/io/
   cat src/lib.vx
   # Check extern "C" declarations
   ```

3. **Create Task Branch:**

   ```bash
   git checkout -b feature/stdlib-integration
   ```

4. **Start with Task 0.1:**
   Update `vex-runtime/c/vex.h` and `vex_io.c` to accept `(ptr, len)` parameters.

---

**Let's build the fastest zero-cost stdlib together! 🎉**
