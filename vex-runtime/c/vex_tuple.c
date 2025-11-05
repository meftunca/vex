/**
 * vex_tuple.c - Tuple<T, U, V> documentation
 *
 * Tuples are COMPILE-TIME ONLY - no runtime implementation needed.
 * Compiler generates struct layout at compile time.
 *
 * Part of Vex Builtin Types - Phase 0
 * Date: November 5, 2025
 */

/*
 * TUPLES ARE COMPILE-TIME STRUCTS
 *
 * The compiler generates a struct with sequential fields for each tuple type.
 *
 * Example Transformations:
 * ========================
 *
 * Vex Code:
 *   let pair: (i32, String) = (42, "hello");
 *
 * Generated LLVM Struct:
 *   %Tuple_i32_String = type { i32, %String }
 *
 * ========================
 *
 * Vex Code:
 *   let triple: (i32, bool, f64) = (1, true, 3.14);
 *
 * Generated LLVM Struct:
 *   %Tuple_i32_bool_f64 = type { i32, i1, double }
 *
 * ========================
 *
 * Field Access:
 * =============
 *
 * Vex Code:
 *   let x = pair.0;  // Access first field
 *   let y = pair.1;  // Access second field
 *
 * Generated LLVM:
 *   %0 = getelementptr inbounds %Tuple_i32_String, %Tuple_i32_String* %pair, i32 0, i32 0
 *   %x = load i32, i32* %0
 *
 *   %1 = getelementptr inbounds %Tuple_i32_String, %Tuple_i32_String* %pair, i32 0, i32 1
 *   %y = load %String, %String* %1
 *
 * ========================
 *
 * Destructuring:
 * ==============
 *
 * Vex Code:
 *   let (a, b, c) = triple;
 *
 * Desugars to:
 *   let a = triple.0;
 *   let b = triple.1;
 *   let c = triple.2;
 *
 * ========================
 *
 * Pattern Matching:
 * =================
 *
 * Vex Code:
 *   match pair {
 *       (0, _) => println("First is zero"),
 *       (x, y) => println("x: ", x, " y: ", y),
 *   }
 *
 * Generated Code:
 *   let tmp_0 = pair.0;
 *   let tmp_1 = pair.1;
 *   if tmp_0 == 0 {
 *       println("First is zero");
 *   } else {
 *       let x = tmp_0;
 *       let y = tmp_1;
 *       println("x: ", x, " y: ", y);
 *   }
 *
 * ========================
 *
 * NO RUNTIME FUNCTIONS NEEDED
 * ============================
 *
 * Tuples are just structs with numbered fields.
 * All operations compile to direct struct field access.
 * Zero runtime overhead - everything is compile-time!
 *
 * Memory Layout:
 * ==============
 *
 * Tuples follow C struct packing rules:
 * - Fields laid out sequentially
 * - Natural alignment for each field
 * - Padding inserted for alignment
 *
 * Example:
 *   (i32, i8, i32) on 64-bit:
 *   [0-3: i32][4: i8][5-7: padding][8-11: i32]
 *   Total size: 12 bytes (with padding)
 *
 * Size Calculation:
 * =================
 *
 * Size = sum of field sizes + padding
 * Alignment = max alignment of all fields
 *
 * Example:
 *   (i32, f64, i8):
 *   Alignment = 8 (from f64)
 *   Size = 4 (i32) + 4 (pad) + 8 (f64) + 1 (i8) + 7 (pad) = 24 bytes
 */

// No functions needed - this file is documentation only!
