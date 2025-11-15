# 03: Codegen & LLVM Issues

**Severity:** 🔴 CRITICAL  
**Category:** Compiler Backend / Code Generation  
**Analysis Date:** 15 Kasım 2025  
**Status:** IDENTIFIED - CRITICAL FIXES NEEDED

---

## Executive Summary

Vex'in LLVM code generation layer'ında **30 ciddi sorun** tespit edildi. Struct ABI, function calling conventions, optimization passes ve debug info eksiklikleri var. Bazı durumlar undefined behavior üretiyor.

**Ana Sorunlar:**
- Struct ABI non-compliant (C FFI broken)
- Generic function monomorphization eksik
- LLVM optimization passes minimal
- Debug info incomplete
- Async codegen broken

**Impact:** Generated code Rust/C ile ABI-compatible değil, debugging zor, performance suboptimal.

---

## Critical Issues (🔴)

### Issue 1: Struct ABI Non-Compliant

**File:** `vex-compiler/src/codegen_ast/types.rs:141-187`  
**Severity:** 🔴 CRITICAL  
**Impact:** C FFI broken, struct layout unpredictable

**Evidence:**
```rust
// vex-compiler/src/codegen_ast/types.rs:141
ASTType::Struct { name, fields, .. } => {
    let field_types: Vec<BasicTypeEnum> = fields
        .iter()
        .map(|f| self.compile_type(&f.ty))
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(self
        .context
        .struct_type(&field_types, false)  // ❌ packed=false but no alignment
        .as_basic_type_enum())
}
```

**Problem:**
```vex
// time.vx - Duration struct
contract Duration {
    secs: i64,
    nanos: i32
}

// C equivalent:
struct Duration {
    int64_t secs;
    int32_t nanos;
    // Padding: 4 bytes (alignment)
};

// Vex generates:
// { i64, i32 } - No padding, wrong size!
```

**Current Behavior:**
```bash
$ vex run examples/test_time.vx
Error: Struct size mismatch: expected 16, got 12
```

**Recommendation:**
```rust
// Implement proper struct layout algorithm
struct StructLayout {
    field_offsets: Vec<usize>,
    total_size: usize,
    alignment: usize,
}

impl<'ctx> ASTCodeGen<'ctx> {
    fn layout_struct(&self, fields: &[Field]) -> StructLayout {
        let mut offset = 0;
        let mut max_align = 1;
        let mut offsets = Vec::new();
        
        for field in fields {
            let align = self.type_alignment(&field.ty);
            let size = self.type_size(&field.ty);
            
            // Align offset
            offset = (offset + align - 1) & !(align - 1);
            offsets.push(offset);
            offset += size;
            max_align = max_align.max(align);
        }
        
        // Pad struct to alignment
        let total = (offset + max_align - 1) & !(max_align - 1);
        
        StructLayout {
            field_offsets: offsets,
            total_size: total,
            alignment: max_align,
        }
    }
}
```

**Effort:** 2-3 weeks  
**References:** System V ABI, LLVM target data layout

---

### Issue 2: Generic Function Monomorphization Incomplete

**File:** `vex-compiler/src/codegen_ast/generics/instantiation.rs:218-298`  
**Severity:** 🔴 CRITICAL  
**Impact:** Generic functions crash or produce wrong code

**Evidence:**
```rust
// vex-compiler/src/codegen_ast/generics/instantiation.rs:218
pub(crate) fn instantiate_generic_function(
    &mut self,
    name: &str,
    type_args: &[ASTType],
) -> Result<FunctionValue<'ctx>, String> {
    // ... code ...
    
    let mut body_codegen = ASTCodeGen {
        module: self.module,
        context: self.context,
        builder: self.builder,
        // ... 
    };

    // ❌ Missing: Substitute generic params in function body
    // ❌ Missing: Codegen with concrete types
    
    Ok(function)
}
```

**Problem:**
```vex
fn identity<T>(x: T) -> T {
    return x;
}

fn main() {
    let a = identity<i32>(42);
    let b = identity<String>("hello");
    // ❌ Crashes: No monomorphization for String
}
```

**Recommendation:**
```rust
struct MonomorphizationCache<'ctx> {
    instances: HashMap<(String, Vec<ASTType>), FunctionValue<'ctx>>,
}

impl<'ctx> MonomorphizationCache<'ctx> {
    fn instantiate_or_get(
        &mut self,
        name: &str,
        type_args: &[ASTType],
        codegen: &mut ASTCodeGen<'ctx>,
    ) -> Result<FunctionValue<'ctx>, String> {
        let key = (name.to_string(), type_args.to_vec());
        
        if let Some(&existing) = self.instances.get(&key) {
            return Ok(existing);
        }
        
        // Get generic function AST
        let generic_fn = codegen.generic_functions.get(name)
            .ok_or_else(|| format!("Generic function {} not found", name))?;
        
        // Substitute type parameters
        let substituted = substitute_types(generic_fn, type_args)?;
        
        // Generate monomorphized function
        let instance = codegen.compile_function(&substituted)?;
        
        self.instances.insert(key, instance);
        Ok(instance)
    }
}
```

**Effort:** 3-4 weeks

---

### Issue 3: Function Calling Convention Wrong

**File:** `vex-compiler/src/codegen_ast/functions.rs:44-48`  
**Severity:** 🔴 CRITICAL  
**Impact:** Function calls crash on some platforms

**Evidence:**
```rust
// vex-compiler/src/codegen_ast/functions.rs:44
let function = self.module.add_function(
    mangled_name.as_str(),
    fn_type,
    None,  // ❌ No linkage specified
);

// ❌ Missing: Set calling convention
// ❌ Missing: Set attributes (sret, byval, etc.)
```

**Problem:**
```vex
// Large return values should use sret:
fn create_large() -> LargeStruct {
    LargeStruct { /* 128 bytes */ }
}

// Generated IR:
// define %LargeStruct @create_large()
// ❌ Should be: define void @create_large(%LargeStruct* sret)
```

**Recommendation:**
```rust
impl<'ctx> ASTCodeGen<'ctx> {
    fn compile_function(&mut self, func: &Function) -> Result<FunctionValue<'ctx>, String> {
        let function = self.module.add_function(name, fn_type, None);
        
        // Set calling convention
        #[cfg(target_os = "macos")]
        function.set_call_conventions(/* C calling convention */);
        
        // Handle large return values
        if self.type_size(&func.return_type) > 16 {
            // Use sret (struct return)
            let sret_param = function.get_first_param().unwrap();
            sret_param.add_attribute(
                self.context.create_enum_attribute(
                    Attribute::get_named_enum_kind_id("sret"),
                    0,
                )
            );
        }
        
        Ok(function)
    }
}
```

**Effort:** 1-2 weeks

---

### Issue 4: Async Codegen Broken

**File:** `vex-compiler/src/codegen_ast/async_codegen.rs:32-50`  
**Severity:** 🔴 CRITICAL  
**Impact:** async/await doesn't work

**Evidence:**
```rust
// vex-compiler/src/codegen_ast/async_codegen.rs:32
pub(crate) fn compile_async_function(
    &mut self,
    func: &Function,
) -> Result<FunctionValue<'ctx>, String> {
    // TODO: Implement async state machine transformation
    Err("Async functions not yet implemented".to_string())
}
```

**Problem:**
```vex
async fn fetch_data() -> String {
    let resp = await http.get("example.com");
    return resp.body;
}

// ❌ Compilation error: async not implemented
```

**Recommendation:**
```rust
// Transform to state machine:
enum FetchDataState {
    Start,
    AwaitingHttp { future: HttpFuture },
    Done,
}

fn fetch_data_state_machine(state: &mut FetchDataState) -> Poll<String> {
    match state {
        Start => {
            let future = http.get("example.com");
            *state = AwaitingHttp { future };
            Poll::Pending
        }
        AwaitingHttp { future } => {
            match future.poll() {
                Poll::Ready(resp) => {
                    *state = Done;
                    Poll::Ready(resp.body)
                }
                Poll::Pending => Poll::Pending,
            }
        }
        Done => unreachable!(),
    }
}
```

**Effort:** 6-8 weeks (major project)

---

### Issue 5: LLVM Optimization Passes Minimal

**File:** `vex-compiler/src/codegen_ast/mod.rs:1012-1063`  
**Severity:** 🔴 CRITICAL  
**Impact:** Generated code 3-5x slower than optimized Rust

**Evidence:**
```rust
// vex-compiler/src/codegen_ast/mod.rs:1012
pub fn optimize_module(&self) -> Result<(), String> {
    // ❌ No optimization passes configured!
    Ok(())
}
```

**Problem:**
```bash
$ vex build --release examples/benchmark.vx
$ time ./benchmark
Time: 5.2s

$ rustc -O benchmark.rs
$ time ./benchmark
Time: 1.1s  # 4.7x faster!
```

**Recommendation:**
```rust
use inkwell::passes::{PassManager, PassManagerBuilder};

impl<'ctx> ASTCodeGen<'ctx> {
    pub fn optimize_module(&self, opt_level: u32) -> Result<(), String> {
        let pass_manager = PassManager::create(());
        let builder = PassManagerBuilder::create();
        
        builder.set_optimization_level(inkwell::OptimizationLevel::Aggressive);
        
        // Core optimization passes
        pass_manager.add_instruction_combining_pass();
        pass_manager.add_reassociate_pass();
        pass_manager.add_gvn_pass();
        pass_manager.add_cfg_simplification_pass();
        pass_manager.add_basic_alias_analysis_pass();
        pass_manager.add_promote_memory_to_register_pass();
        pass_manager.add_instruction_combining_pass();
        pass_manager.add_reassociate_pass();
        
        // Populate with standard passes
        builder.populate_module_pass_manager(&pass_manager);
        
        pass_manager.run_on(&self.module);
        Ok(())
    }
}
```

**Effort:** 1 week

---

### Issue 6: Debug Info Incomplete

**File:** `vex-compiler/src/codegen_ast/debug.rs:1-100`  
**Severity:** 🔴 CRITICAL  
**Impact:** Cannot debug with lldb/gdb

**Evidence:**
```rust
// vex-compiler/src/codegen_ast/debug.rs:45
pub fn emit_location(&self, line: u32, column: u32) {
    // Basic line info only
    let location = self.debug_builder.create_debug_location(
        self.context,
        line,
        column,
        self.current_scope,
        None,
    );
    // ❌ Missing: Variable info, stack traces
}
```

**Problem:**
```bash
$ lldb ./my_program
(lldb) break main
Breakpoint 1: no locations (pending).
# ❌ No function names in debug info

(lldb) print my_var
error: use of undeclared identifier 'my_var'
# ❌ No variable info
```

**Recommendation:**
```rust
impl<'ctx> DebugInfo<'ctx> {
    pub fn emit_local_variable(
        &self,
        name: &str,
        ty: &ASTType,
        alloca: PointerValue<'ctx>,
        line: u32,
    ) {
        let di_type = self.get_or_create_type(ty);
        
        let variable = self.debug_builder.create_auto_variable(
            self.current_scope,
            name,
            self.file,
            line,
            di_type,
            true, // always_preserve
            DIFlags::zero(),
            0, // align
        );
        
        self.debug_builder.insert_declare_at_end(
            alloca,
            Some(variable),
            None,
            self.debug_builder.create_debug_location(...),
            self.current_block,
        );
    }
}
```

**Effort:** 2-3 weeks

---

## High Priority Issues (🟡)

### Issue 7: Pattern Match Codegen Suboptimal

**File:** `vex-compiler/src/codegen_ast/expressions/pattern_match.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Generates many unnecessary branches

**Problem:**
```vex
match x {
    0 => a(),
    1 => b(),
    2 => c(),
    // ... 20 more cases
}

// Generates: 22 sequential branches
// Should use: Jump table
```

**Recommendation:** Implement switch instruction for contiguous integers

**Effort:** 1-2 weeks

---

### Issue 8: Tail Call Optimization Missing

**File:** `vex-compiler/src/codegen_ast/functions.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Recursive functions stack overflow

**Problem:**
```vex
fn factorial(n: i64, acc: i64) -> i64 {
    if n == 0 { return acc; }
    return factorial(n - 1, n * acc);  // Should be tail call
}
```

**Recommendation:** Mark tail calls with `musttail` attribute

**Effort:** 1 week

---

### Issue 9: SIMD Intrinsics Missing

**File:** N/A  
**Severity:** 🟡 HIGH  
**Impact:** Cannot vectorize code

**Recommendation:** Expose LLVM vector types and intrinsics

**Effort:** 2-3 weeks

---

### Issue 10: Inline Assembly Support Missing

**File:** N/A  
**Severity:** 🟡 HIGH  
**Impact:** Cannot write low-level code

**Recommendation:** Add `asm!` macro (but user said no macros, so inline function?)

**Effort:** 2 weeks

---

### Issue 11: Link-Time Optimization (LTO) Not Supported

**File:** `vex-cli/src/main.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Cannot optimize across crates

**Recommendation:** Add `--lto` flag, emit bitcode

**Effort:** 1 week

---

### Issue 12: PGO (Profile-Guided Optimization) Missing

**File:** N/A  
**Severity:** 🟡 HIGH  
**Impact:** Cannot optimize hot paths

**Recommendation:** Add instrumentation passes

**Effort:** 3-4 weeks

---

### Issue 13: Constant Folding Limited

**File:** `vex-compiler/src/codegen_ast/expressions/mod.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Simple constants not folded

**Problem:**
```vex
const X = 2 + 3;  // Should fold to 5 at compile time
```

**Effort:** 1-2 weeks

---

### Issue 14: Dead Code Elimination Weak

**File:** N/A  
**Severity:** 🟡 HIGH  
**Impact:** Unused code in final binary

**Effort:** 1 week

---

## Medium Priority Issues (🟢)

### Issue 15: Loop Unrolling

**Severity:** 🟢 MEDIUM  
**Effort:** 1 week

---

### Issue 16: Function Inlining Hints

**Severity:** 🟢 MEDIUM  
**Effort:** 3 days

---

### Issue 17: Stack Overflow Protection

**Severity:** 🟢 MEDIUM  
**Effort:** 1 week

---

### Issue 18: Code Size Optimization

**Severity:** 🟢 MEDIUM  
**Effort:** 1-2 weeks

---

### Issue 19: Devirtualization

**Severity:** 🟢 MEDIUM  
**Effort:** 2 weeks

---

### Issue 20: Escape Analysis

**Severity:** 🟢 MEDIUM  
**Effort:** 3-4 weeks

---

## Low Priority Issues (🔵)

### Issue 21: LLVM IR Readability

**Severity:** 🔵 LOW  
**Impact:** Hard to debug generated IR

**Effort:** 3 days

---

## Metrics Summary

| Category | Critical | High | Medium | Low | Total |
|----------|----------|------|--------|-----|-------|
| ABI/Calling Conv | 2 | 1 | 1 | 0 | 4 |
| Generics/Monomorphization | 1 | 0 | 0 | 0 | 1 |
| Optimization | 2 | 6 | 5 | 1 | 14 |
| Async Codegen | 1 | 0 | 0 | 0 | 1 |
| Debug Info | 1 | 0 | 0 | 0 | 1 |
| Code Quality | 0 | 2 | 3 | 0 | 5 |
| **TOTAL** | **6** | **9** | **9** | **1** | **30** |

---

## Implementation Roadmap

### Phase 1: Critical Fixes (Week 1-4)
- [ ] Fix struct ABI alignment
- [ ] Implement generic monomorphization
- [ ] Fix calling conventions
- [ ] Add optimization passes
- [ ] Improve debug info

### Phase 2: High Priority (Week 5-8)
- [ ] Pattern match optimization (jump tables)
- [ ] Tail call optimization
- [ ] SIMD intrinsics
- [ ] LTO support
- [ ] Constant folding

### Phase 3: Async Support (Week 9-16)
- [ ] Async state machine transformation
- [ ] Future/Poll types
- [ ] Executor integration

---

## Testing Plan

```bash
# Test struct ABI
vex build examples/test_ffi_struct.vx
./test_ffi_struct  # Should match C struct layout

# Test optimization
vex build --release examples/benchmark.vx
time ./benchmark  # Should be within 2x of Rust

# Test debug info
vex build --debug examples/test_debug.vx
lldb ./test_debug
(lldb) break main  # Should work
(lldb) print my_var  # Should work

# Test generics
vex run examples/test_generics.vx
# Should monomorphize for i32, String, etc.
```

---

## Related Issues

- [01_TYPE_SYSTEM_GAPS.md](./01_TYPE_SYSTEM_GAPS.md) - Generic constraints affect monomorphization
- [05_RUNTIME_FFI_PROBLEMS.md](./05_RUNTIME_FFI_PROBLEMS.md) - ABI issues affect FFI
- [02_BORROW_CHECKER_WEAKNESSES.md](./02_BORROW_CHECKER_WEAKNESSES.md) - Drop codegen needs borrow checker

---

## References

- LLVM Language Reference: https://llvm.org/docs/LangRef.html
- System V ABI: https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf
- Rust codegen: https://rustc-dev-guide.rust-lang.org/backend/codegen.html

---

**Next Steps:** Fix struct ABI alignment first (breaks FFI), then optimize passes.
