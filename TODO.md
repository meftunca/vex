# Vex Language Development TODO

## 🔧 Generic Type System - Stability Improvements

**Status**: Partially Fixed (443/447 tests passing - 99.1%)  
**Priority**: Medium  
**Impact**: Edge cases in complex generic scenarios

### Current Implementation (Working but Not Optimal)

✅ **What's Working:**

- AST-level type substitution via `substitute_types_in_function`
- Parameter types correctly substituted (T → I32)
- Return type inference for instantiated generics
- Basic generic functions work correctly

⚠️ **Workarounds in Place:**

1. **Generic Parameter Type Inference Skip**

   - Location: `vex-compiler/src/codegen_ast/expressions/calls/function_calls.rs:171-179`
   - Issue: Generic function parameters skipped during type inference to avoid Named("T") errors
   - Code: `let is_generic = !func_def.type_params.is_empty();`
   - Risk: Type information loss in complex scenarios

2. **Return Type Naming Convention Dependency**

   - Location: `vex-compiler/src/codegen_ast/types/inference.rs:605-620`
   - Issue: Tries `min_i32` (type param based) fallback for `min_i32_i32` (arg based)
   - Code: `let generic_name = format!("{}{}", func_name, first_arg_suffix);`
   - Risk: Fragile if naming conventions change

3. **Unused Type Substitution Context**
   - Location: `vex-compiler/src/codegen_ast/struct_def.rs:65-71`
   - Field: `active_type_substitutions: HashMap<String, Type>`
   - Issue: Saved/restored but never actually used for Ident resolution
   - Status: Dead code that should be removed or properly utilized

### Failing Tests (4 remaining)

1. `test_generic_overload` - println type mismatch (separate bug)
2. `test_generics_comprehensive` - May be related
3. `test_trait_based_cleanup` - Trait system issue
4. `05_generics/nested_extreme` - Edge case

### Recommended Clean Implementation

**Goal**: Eliminate workarounds, achieve 100% Rust-level stability

**Approach**:

1. Remove `active_type_substitutions` dead code OR implement proper usage
2. Unify function naming: Use consistent convention for both generic instantiation and overload resolution
3. Add proper type context to generic parameter inference instead of skipping
4. Test edge cases:
   - Nested generics: `Vec<Vec<T>>`
   - Multi-param generics: `HashMap<K, V>`
   - Generic methods: `Container<T>.map<U>()`
   - Generic traits: `impl<T> Trait for Type<T>`

**Estimated Effort**: 2-3 hours for clean refactor

---
