# Vex Architecture - Critical Bugs & Systematic Solutions

**Date:** 14 Kasım 2025  
**Status:** ANALYSIS & ACTION PLAN  
**Priority:** CRITICAL - Foundation System

---

## 🔴 PROBLEM STATEMENT

Stdlib development is blocked by systematic issues in generic type system and method instantiation. Current approach uses quick fixes that accumulate technical debt and cause cascading failures.

**Symptom:** 16 failing tests, runtime crashes, linking errors, type inference failures  
**Root Cause:** Incomplete generic method instantiation pipeline + missing type propagation  
**Impact:** Cannot ship Layer 2 stdlib (Vec, Box, Option, Result)

---

## 📊 ROOT CAUSE ANALYSIS

### 1. Type Inference Pipeline Gaps

**Current State:**

```rust
// vex-compiler/src/codegen_ast/generics/inference.rs
pub(crate) fn infer_type_args_from_call(
    &mut self,
    func_def: &Function,
    args: &[Expression],
) -> Result<Vec<Type>, String> {
    // Only infers from first argument!
    let first_arg_type = self.infer_expression_type(&args[0])?;

    // Assumes ALL type params = first arg type
    // BROKEN for multi-param generics
    let mut type_args = Vec::new();
    for _ in 0..func_def.type_params.len() {
        type_args.push(first_arg_type.clone()); // ❌ WRONG
    }
    Ok(type_args)
}
```

**Problems:**

- ❌ No receiver type propagation (`v.push(10)` doesn't know `v: Vec<i32>`)
- ❌ No constraint solving (`HashMap<K,V>` cannot infer both K and V)
- ❌ No backward inference (return type doesn't propagate)
- ❌ No unification algorithm (cannot solve `T = i32` from context)

**Test Failures:**

- `vec_full_test.vx`: `Vec()` → cannot infer `T`
- `vec_constructor_test.vx`: same issue
- `nested_generics.vx`: multi-level generic resolution fails

---

### 2. Method Call Type Propagation

**Current State:**

```rust
// vex-compiler/src/codegen_ast/expressions/calls/method_calls.rs:95
let receiver_type = self.infer_expression_type(receiver)?;
let struct_type_args = self.extract_type_args_from_type(&receiver_type)?;

// Later, at instantiation:
if method_resolution_result.is_err() {
    // Only extracts from mangled name, not receiver type!
    let parts: Vec<&str> = struct_name.split('_').collect();
    let base_struct_name = parts[0]; // ❌ LOSES type args
}
```

**Problems:**

- ❌ Type args extracted but not used consistently
- ❌ Mangled name parsing fragile (`Vec_i32` vs `HashMap_str_i32`)
- ❌ First method call has no instantiated methods yet
- ❌ Recursive dependency: need type to instantiate, need instantiation to get type

**Test Failures:**

- `test_vec_pure.vx`: First `v.push(10)` generates `Vec_push` instead of `Vec_i32_push`
- Linking fails: `Undefined symbols: _Vec_push_i32`

---

### 3. Generic Method Instantiation Order

**Current State:**

```rust
// On first method call:
🔍 Method call Vec.push() with 1 args
🔍 Checking method names:
   external_method_typed: Vec_push_i32_1 (exists: false)
   external_method_name: Vec_push (exists: false)  ❌ Not instantiated yet!

🔍 Attempting generic method instantiation for Vec.push
  📥 Provided type args: []  ❌ Empty! Should be [I32]
  ⚠️  WARNING: No type arguments provided for generic struct!
```

**Problems:**

- ❌ Method instantiation happens DURING call compilation
- ❌ Type args not available when needed
- ❌ Instantiation generates wrong mangled name (`Vec_push` vs `Vec_i32_push`)
- ❌ Subsequent calls find wrong method, cascade failure

**Test Failures:**

- All Vec tests fail on first method call
- Linking errors due to symbol mismatch

---

### 4. Struct Instantiation vs Method Instantiation

**Current State:**

```rust
// Struct instantiation works:
let v: Vec<i32> = vec_new<i32>();  ✅
// → Instantiates Vec_i32 struct
// → Calls instantiate_struct_methods()
// → Generates Vec_i32_push, Vec_i32_get, etc.

// Method call works:
v.push(10);  ✅
// → Finds Vec_i32_push in function table

// But constructor inference broken:
let v = Vec();  ❌
// → Cannot infer type, defaults to i32
// → Instantiates Vec struct but methods wrong
v.push(10);  ❌
// → Tries to instantiate Vec_push (no type suffix)
```

**Problems:**

- ❌ No bidirectional type flow (constructor ← first method call)
- ❌ Struct instantiation and method instantiation decoupled
- ❌ Type inference happens too late (during method call, not during struct creation)

---

## 🎯 SYSTEMATIC SOLUTION PLAN

### Phase 1: Type Inference Foundation (CORE FIX)

**Goal:** Proper type propagation through compilation pipeline

**Implementation:**

1. **Add Type Context to Compilation State**

   ```rust
   // New field in ASTCodeGen
   pub struct ASTCodeGen<'ctx> {
       // ... existing fields

       /// Variable type information (name → concrete type)
       variable_types: HashMap<String, Type>,

       /// Expression type cache (avoid re-inference)
       expr_type_cache: HashMap<usize, Type>, // hash of expr → type

       /// Pending type constraints (for unification)
       type_constraints: Vec<TypeConstraint>,
   }

   pub enum TypeConstraint {
       Equal(Type, Type),           // T = i32
       MethodReceiver(String, Type), // receiver.method → receiver: Type
       Assignment(String, Type),     // var = expr → var: Type
   }
   ```

2. **Enhance Type Inference**

   ```rust
   // vex-compiler/src/codegen_ast/types/inference.rs (NEW FILE)
   impl<'ctx> ASTCodeGen<'ctx> {
       /// Infer expression type with context propagation
       pub fn infer_expression_type_with_context(
           &mut self,
           expr: &Expression,
           expected_type: Option<&Type>,
       ) -> Result<Type, String> {
           match expr {
               Expression::Ident(name) => {
                   // Check variable_types first
                   if let Some(ty) = self.variable_types.get(name) {
                       return Ok(ty.clone());
                   }
                   // Fallback to existing logic
                   self.infer_expression_type(expr)
               }

               Expression::Call { func, type_args, args, .. } => {
                   if let Expression::Ident(name) = func.as_ref() {
                       // Constructor call: Vec<i32>() or Vec()
                       if self.struct_ast_defs.contains_key(name) {
                           if type_args.is_empty() {
                               // Try to infer from expected_type
                               if let Some(Type::Generic { name: struct_name, type_args: expected_args }) = expected_type {
                                   if struct_name == name {
                                       return Ok(expected_type.unwrap().clone());
                                   }
                               }

                               // Try to infer from first usage (deferred)
                               return Ok(Type::Generic {
                                   name: name.clone(),
                                   type_args: vec![Type::Unknown], // Placeholder
                               });
                           }
                       }
                   }

                   self.infer_expression_type(expr)
               }

               _ => self.infer_expression_type(expr),
           }
       }

       /// Resolve Type::Unknown to concrete types using constraints
       pub fn unify_types(&mut self) -> Result<(), String> {
           // Simple unification for now
           for constraint in &self.type_constraints {
               match constraint {
                   TypeConstraint::Equal(Type::Unknown, concrete) => {
                       // Replace all Unknown with concrete
                       self.apply_type_substitution(&Type::Unknown, concrete)?;
                   }
                   _ => {}
               }
           }
           Ok(())
       }
   }
   ```

3. **Variable Declaration Type Tracking**
   ```rust
   // In compile_let_statement
   fn compile_let_statement(&mut self, ...) -> Result<...> {
       // ... existing code

       // Register variable type
       let final_type = if let Some(explicit_type) = ty {
           explicit_type.clone()
       } else {
           self.infer_expression_type_with_context(value, None)?
       };

       self.variable_types.insert(name.clone(), final_type.clone());

       // If type has Unknown, add constraint
       if self.contains_unknown(&final_type) {
           self.type_constraints.push(TypeConstraint::Assignment(
               name.clone(),
               final_type,
           ));
       }

       // ... rest of compilation
   }
   ```

**Tests (Rust unit tests FIRST):**

```rust
// vex-compiler/tests/type_inference_test.rs
#[test]
fn test_variable_type_tracking() {
    let mut codegen = setup_codegen();

    // let v: Vec<i32> = vec_new<i32>();
    codegen.variable_types.insert("v".to_string(), Type::Generic {
        name: "Vec".to_string(),
        type_args: vec![Type::I32],
    });

    // v.push(10) should infer Vec<i32>
    let receiver = Expression::Ident("v".to_string());
    let ty = codegen.infer_expression_type(&receiver).unwrap();
    assert_eq!(ty, Type::Generic { name: "Vec".to_string(), type_args: vec![Type::I32] });
}

#[test]
fn test_method_receiver_type_propagation() {
    // v.push(10) → should know v: Vec<i32>
    // Then instantiate Vec_i32_push
}

#[test]
fn test_constructor_inference_deferred() {
    // let v = Vec(); v.push(10);
    // Should infer Vec<i32> from first method call
}
```

**Expected Outcome:**

- ✅ Variable types tracked in compilation state
- ✅ Type inference uses context
- ✅ Deferred type resolution with constraints

---

### Phase 2: Method Instantiation Pipeline (FIX INSTANTIATION)

**Goal:** Instantiate methods with correct type arguments BEFORE first call

**Implementation:**

1. **Eagerly Instantiate Methods When Receiver Type Known**

   ```rust
   // vex-compiler/src/codegen_ast/expressions/calls/method_calls.rs
   pub(crate) fn compile_method_call(
       &mut self,
       receiver: &Expression,
       method: &str,
       type_args: &[Type],
       args: &[Expression],
   ) -> Result<BasicValueEnum<'ctx>, String> {
       // Get receiver type (use variable_types for Ident)
       let receiver_type = if let Expression::Ident(name) = receiver {
           self.variable_types.get(name)
               .cloned()
               .ok_or_else(|| format!("Unknown variable: {}", name))?
       } else {
           self.infer_expression_type_with_context(receiver, None)?
       };

       eprintln!("🔍 Receiver type from context: {:?}", receiver_type);

       // Extract struct name and type args
       let (base_struct_name, struct_type_args) = match &receiver_type {
           Type::Generic { name, type_args } => (name.clone(), type_args.clone()),
           Type::Named(name) => (name.clone(), vec![]),
           _ => return Err(format!("Cannot call method on type: {:?}", receiver_type)),
       };

       // Check if method is already instantiated
       let mangled_method_name = self.build_mangled_method_name(
           &base_struct_name,
           &struct_type_args,
           method,
       );

       if !self.functions.contains_key(&mangled_method_name) {
           // Method not instantiated yet - instantiate now!
           eprintln!("🔧 Method not instantiated, instantiating: {}", mangled_method_name);

           if let Ok(method_def) = self.find_generic_method(&base_struct_name, method) {
               self.instantiate_generic_method(
                   &base_struct_name,
                   &struct_type_args,
                   method,
                   &method_def,
                   &[], // arg_types not needed if struct_type_args complete
               )?;
           } else {
               return Err(format!(
                   "Method '{}' not found for struct '{}'",
                   method, base_struct_name
               ));
           }
       }

       // Now compile the method call (method is guaranteed to exist)
       // ... existing call compilation logic
   }

   fn build_mangled_method_name(
       &self,
       struct_name: &str,
       type_args: &[Type],
       method_name: &str,
   ) -> String {
       if type_args.is_empty() {
           format!("{}_{}", struct_name, method_name)
       } else {
           let type_names: Vec<String> = type_args
               .iter()
               .map(|t| self.type_to_string(t))
               .collect();
           format!("{}_{}_{}",  struct_name, type_names.join("_"), method_name)
       }
   }
   ```

2. **Fix Mangled Name Generation Consistency**
   ```rust
   // Ensure all paths use same mangling logic
   // 1. Struct instantiation: Vec_i32
   // 2. Method instantiation: Vec_i32_push
   // 3. Method lookup: Vec_i32_push
   // All must use build_mangled_method_name()
   ```

**Tests (Rust unit tests):**

```rust
#[test]
fn test_method_instantiation_before_call() {
    let mut codegen = setup_codegen();

    // Register Vec<i32>
    codegen.variable_types.insert("v".to_string(), Type::Generic {
        name: "Vec".to_string(),
        type_args: vec![Type::I32],
    });

    // First call should instantiate Vec_i32_push
    let receiver = Expression::Ident("v".to_string());
    let args = vec![Expression::IntLiteral(10)];

    codegen.compile_method_call(&receiver, "push", &[], &args).unwrap();

    // Check that Vec_i32_push exists
    assert!(codegen.functions.contains_key("Vec_i32_push"));
}

#[test]
fn test_mangled_name_consistency() {
    let codegen = setup_codegen();

    let name1 = codegen.build_mangled_method_name("Vec", &[Type::I32], "push");
    let name2 = codegen.build_mangled_method_name("Vec", &[Type::I32], "push");

    assert_eq!(name1, name2);
    assert_eq!(name1, "Vec_i32_push");
}
```

**Expected Outcome:**

- ✅ Methods instantiated when receiver type known
- ✅ Consistent mangled names across all paths
- ✅ First method call succeeds

---

### Phase 3: Constructor Type Inference (BIDIRECTIONAL FLOW)

**Goal:** `let v = Vec(); v.push(10);` should infer `Vec<i32>` from first usage

**Implementation:**

1. **Two-Pass Compilation for Type Inference**

   ```rust
   // vex-compiler/src/codegen_ast/statements/mod.rs
   fn compile_block(&mut self, statements: &[Statement]) -> Result<...> {
       // PASS 1: Collect type constraints
       for stmt in statements {
           self.collect_type_constraints(stmt)?;
       }

       // RESOLVE: Unify types
       self.unify_types()?;

       // PASS 2: Compile with resolved types
       for stmt in statements {
           self.compile_statement(stmt)?;
       }
   }

   fn collect_type_constraints(&mut self, stmt: &Statement) -> Result<(), String> {
       match stmt {
           Statement::Let { name, ty, value, .. } => {
               if ty.is_none() {
                   // Inferred type variable
                   let var_type = Type::Unknown;
                   self.variable_types.insert(name.clone(), var_type.clone());

                   // Add constraint from value
                   let value_type = self.infer_expression_type_with_context(value, None)?;
                   self.type_constraints.push(TypeConstraint::Equal(var_type, value_type));
               }
           }

           Statement::Expression(Expression::MethodCall { receiver, method, args, .. }) => {
               // Add constraint: receiver type must support method with these arg types
               let receiver_type = if let Expression::Ident(name) = receiver.as_ref() {
                   self.variable_types.get(name).cloned().unwrap_or(Type::Unknown)
               } else {
                   Type::Unknown
               };

               // Infer from args if receiver unknown
               if matches!(receiver_type, Type::Unknown | Type::Generic { type_args: ref args, .. } if args.iter().any(|t| matches!(t, Type::Unknown))) {
                   let arg_types: Vec<Type> = args.iter()
                       .map(|arg| self.infer_expression_type(arg))
                       .collect::<Result<Vec<_>, _>>()?;

                   self.type_constraints.push(TypeConstraint::MethodReceiver(
                       receiver.to_string(), // placeholder
                       self.infer_receiver_type_from_method(method, &arg_types)?
                   ));
               }
           }

           _ => {}
       }
       Ok(())
   }

   fn infer_receiver_type_from_method(
       &self,
       method: &str,
       arg_types: &[Type],
   ) -> Result<Type, String> {
       // Look up method signature in function_defs
       // For Vec.push(i32), infer Vec<i32>

       for (func_name, func_def) in &self.function_defs {
           if func_name.ends_with(&format!("_{}", method)) {
               // Check if params match
               if func_def.params.len() == arg_types.len() {
                   // Extract struct name and type params
                   // Vec_push → Vec<T> where T comes from first param type

                   // For now, simple heuristic:
                   if method == "push" && arg_types.len() == 1 {
                       // Vec<T>::push(value: T)
                       let struct_name = func_name.strip_suffix(&format!("_{}", method)).unwrap();
                       return Ok(Type::Generic {
                           name: struct_name.to_string(),
                           type_args: vec![arg_types[0].clone()],
                       });
                   }
               }
           }
       }

       Err(format!("Cannot infer receiver type for method '{}'", method))
   }
   ```

2. **Constraint Unification**
   ```rust
   fn unify_types(&mut self) -> Result<(), String> {
       let mut changed = true;
       while changed {
           changed = false;

           for constraint in self.type_constraints.clone() {
               match constraint {
                   TypeConstraint::Equal(Type::Unknown, concrete) |
                   TypeConstraint::Equal(concrete, Type::Unknown) => {
                       // Substitute Unknown with concrete in all variable_types
                       for (_, var_type) in self.variable_types.iter_mut() {
                           if matches!(var_type, Type::Unknown) {
                               *var_type = concrete.clone();
                               changed = true;
                           }
                       }
                   }

                   TypeConstraint::MethodReceiver(receiver_expr, inferred_type) => {
                       // Extract variable name from receiver_expr
                       if let Some(var_name) = self.extract_var_name(&receiver_expr) {
                           if let Some(var_type) = self.variable_types.get_mut(&var_name) {
                               if let Type::Generic { type_args, .. } = var_type {
                                   // Unify type args
                                   if let Type::Generic { type_args: inferred_args, .. } = &inferred_type {
                                       for (i, arg) in type_args.iter_mut().enumerate() {
                                           if matches!(arg, Type::Unknown) && i < inferred_args.len() {
                                               *arg = inferred_args[i].clone();
                                               changed = true;
                                           }
                                       }
                                   }
                               }
                           }
                       }
                   }

                   _ => {}
               }
           }
       }

       // Check for remaining Unknown types
       for (name, ty) in &self.variable_types {
           if self.contains_unknown(ty) {
               return Err(format!(
                   "Cannot infer type for variable '{}': {:?}",
                   name, ty
               ));
           }
       }

       Ok(())
   }
   ```

**Tests (Rust unit tests):**

```rust
#[test]
fn test_bidirectional_inference() {
    let mut codegen = setup_codegen();

    // let v = Vec();
    codegen.variable_types.insert("v".to_string(), Type::Generic {
        name: "Vec".to_string(),
        type_args: vec![Type::Unknown],
    });

    // v.push(10)
    codegen.type_constraints.push(TypeConstraint::MethodReceiver(
        "v".to_string(),
        Type::Generic {
            name: "Vec".to_string(),
            type_args: vec![Type::I32],
        },
    ));

    // Unify
    codegen.unify_types().unwrap();

    // Check that v is now Vec<i32>
    let v_type = codegen.variable_types.get("v").unwrap();
    assert_eq!(v_type, &Type::Generic {
        name: "Vec".to_string(),
        type_args: vec![Type::I32],
    });
}

#[test]
fn test_constructor_inference_from_method() {
    // Complete integration test
    // let v = Vec(); v.push(10); let x = v.len();
    // Should infer Vec<i32> and compile successfully
}
```

**Expected Outcome:**

- ✅ `Vec()` creates placeholder `Vec<Unknown>`
- ✅ First method call infers `Unknown → i32`
- ✅ Unification resolves all types
- ✅ Methods instantiated with correct types

---

### Phase 4: Runtime Stability (MEMORY SAFETY)

**Goal:** Fix trace trap in `test_vec_pure` (malloc/realloc/println interaction)

**Implementation:**

1. **Add Runtime Validation**

   ```c
   // vex-runtime/c/vex_alloc.c
   void* vex_malloc_checked(size_t size) {
       if (size == 0) {
           fprintf(stderr, "WARNING: malloc(0) called\n");
           return NULL;
       }

       void* ptr = malloc(size);
       if (!ptr) {
           fprintf(stderr, "FATAL: malloc(%zu) failed\n", size);
           abort();
       }

       // Debug: Track allocation
       #ifdef VEX_DEBUG_ALLOC
       fprintf(stderr, "ALLOC: %p (%zu bytes)\n", ptr, size);
       #endif

       return ptr;
   }

   void* vex_realloc_checked(void* ptr, size_t new_size) {
       if (new_size == 0) {
           fprintf(stderr, "WARNING: realloc(%p, 0) called\n", ptr);
           free(ptr);
           return NULL;
       }

       #ifdef VEX_DEBUG_ALLOC
       fprintf(stderr, "REALLOC: %p → %zu bytes\n", ptr, new_size);
       #endif

       void* new_ptr = realloc(ptr, new_size);
       if (!new_ptr) {
           fprintf(stderr, "FATAL: realloc(%p, %zu) failed\n", ptr, new_size);
           abort();
       }

       #ifdef VEX_DEBUG_ALLOC
       fprintf(stderr, "REALLOC: → %p\n", new_ptr);
       #endif

       return new_ptr;
   }
   ```

2. **Update Vec to Use Checked Allocators**

   ```vex
   // stdlib/core/src/vec.vx
   fn (self: &Vec<T>!) grow() {
       let! new_cap: i64 = 0;
       if self.cap == 0 {
           new_cap = 4;
       } else {
           new_cap = self.cap * 2;
       }

       let elem_size = sizeof<T>();
       let new_size = new_cap * elem_size;

       // Add bounds check
       if new_size <= 0 || new_size > 1_000_000_000 {  // 1GB limit
           panic("Vec::grow: invalid size");
       }

       if self.cap == 0 {
           self.data = malloc(new_size) as *T;
       } else {
           self.data = realloc(self.data as ptr, new_size) as *T;
       }

       // Verify allocation succeeded
       if self.data as i64 == 0 {
           panic("Vec::grow: allocation failed");
       }

       self.cap = new_cap;
   }
   ```

3. **Debug Test with Instrumentation**

   ```bash
   # Compile with debug allocator
   VEX_DEBUG_ALLOC=1 vex compile examples/test_vec_pure.vx

   # Run under lldb to catch exact crash point
   lldb vex-builds/test_vec_pure
   (lldb) run
   (lldb) bt  # backtrace at crash
   ```

**Tests (C unit tests + integration):**

```c
// vex-runtime/tests/test_alloc.c
void test_malloc_zero() {
    void* ptr = vex_malloc_checked(0);
    assert(ptr == NULL);
}

void test_realloc_null() {
    void* ptr = vex_realloc_checked(NULL, 100);
    assert(ptr != NULL);
    free(ptr);
}

void test_vec_grow_sequence() {
    // Simulate Vec growth: 0 → 4 → 8 → 16
    void* ptr = NULL;
    ptr = vex_malloc_checked(4 * sizeof(int));
    ptr = vex_realloc_checked(ptr, 8 * sizeof(int));
    ptr = vex_realloc_checked(ptr, 16 * sizeof(int));
    free(ptr);
}
```

**Expected Outcome:**

- ✅ Malloc/realloc failures caught with clear errors
- ✅ Debug instrumentation shows allocation sequence
- ✅ Trace trap root cause identified
- ✅ `test_vec_pure` runs without crash

---

## 📝 DEVELOPMENT WORKFLOW

### 1. Rust Unit Tests FIRST

```bash
# Test type inference
cargo test --package vex-compiler test_type_inference

# Test method instantiation
cargo test --package vex-compiler test_method_instantiation

# Test constraint solving
cargo test --package vex-compiler test_constraint_unification
```

### 2. Integration Tests (Vex files) SECOND

```bash
# Simple cases
vex compile tests/generics/simple_inference.vx
vex run tests/generics/simple_inference.vx

# Complex cases
vex compile examples/test_vec_pure.vx
vex run examples/test_vec_pure.vx
```

### 3. Full Test Suite LAST

```bash
./test_all.sh
```

---

## 🎯 SUCCESS CRITERIA

**Phase 1 Complete:**

- [ ] All Rust unit tests pass (type inference)
- [ ] `variable_types` HashMap tracks all variables
- [ ] Type constraints collected correctly

**Phase 2 Complete:**

- [ ] All Rust unit tests pass (method instantiation)
- [ ] First method call generates correct mangled name
- [ ] `vec_full_test.vx` compiles (may not run yet)

**Phase 3 Complete:**

- [ ] Bidirectional type flow works
- [ ] `let v = Vec(); v.push(10);` compiles
- [ ] All 16 failing tests compile

**Phase 4 Complete:**

- [ ] `test_vec_pure` runs without crash
- [ ] All Vec tests pass
- [ ] No memory leaks (valgrind clean)

**Final Validation:**

- [ ] All 16 tests pass
- [ ] Layer 2 stdlib complete (Vec, Box, Option, Result)
- [ ] Documentation updated
- [ ] No regressions in existing tests

---

## ⏱️ ESTIMATED TIMELINE

**Phase 1:** 4-6 hours (type inference foundation)  
**Phase 2:** 3-4 hours (method instantiation pipeline)  
**Phase 3:** 4-5 hours (constructor inference)  
**Phase 4:** 2-3 hours (runtime stability)

**Total:** 13-18 hours (2-3 days of focused work)

---

## 🚫 ANTI-PATTERNS TO AVOID

1. ❌ **No quick fixes** - Fix root cause, not symptoms
2. ❌ **No defaults** - Don't default to i32 when type unknown
3. ❌ **No silent failures** - Fail loudly with clear errors
4. ❌ **No skip logic** - Don't skip type checking/inference
5. ❌ **Test Vex files first** - Always Rust tests first!

---

## 📚 REFERENCES

**Related Code:**

- `vex-compiler/src/codegen_ast/generics/inference.rs`
- `vex-compiler/src/codegen_ast/generics/methods.rs`
- `vex-compiler/src/codegen_ast/expressions/calls/method_calls.rs`
- `stdlib/core/src/vec.vx`

**Design Docs:**

- `docs/ARCHITECTURE.md`
- `docs/PROJECT_STATUS.md`
- `Specifications/type-system-spec.md`

---

**Status:** READY FOR IMPLEMENTATION  
**Next Step:** Get approval, then start Phase 1
