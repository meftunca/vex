// Codegen Strategy for Generic Impl Clause
//
// Problem: struct Vector impl Add<i32>, Add<f64>, Mul<i32>
// Has 3 operator methods but they need different mangling
//
// Current: All methods named "Vector_op+_1" (same name = collision!)
// Need: "Vector_Add_i32_op+_1", "Vector_Add_f64_op+_1", "Vector_Mul_i32_op*_1"
//
// Solution:
// 1. When declaring/compiling struct methods, check parameter types
// 2. Match param types against trait impl type args
// 3. Mangle method name with trait name + type args
//
// Example:
//   fn (self: &Vector!) op+(other: i32): Vector  → Vector_Add_i32_op+_1
//   fn (self: &Vector!) op+(other: f64): Vector  → Vector_Add_f64_op+_1
//   fn (self: &Vector!) op*(other: i32): Vector  → Vector_Mul_i32_op*_1
//
// Method call lookup:
//   v + 5 → compile_binary_op → compile_method_call
//   → Infer arg type (i32)
//   → Lookup "Vector_Add_i32_op+_1"
//
// Implementation Steps:
// 1. Update declare_struct_method() - match param types to trait impls
// 2. Update compile_struct_method() - use same mangling
// 3. Update compile_method_call() - infer type args from actual args
// 4. Update binary_ops - pass arg type info to method lookup
