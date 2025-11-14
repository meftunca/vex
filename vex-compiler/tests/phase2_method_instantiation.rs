// Phase 2: Method Instantiation Pipeline - Unit Tests
// Test receiver type resolution, mangled name generation, eager instantiation

use vex_ast::*;
use vex_compiler::codegen_ast;

#[cfg(test)]
mod phase2_method_instantiation_tests {
    use super::*;
    use inkwell::context::Context;

    fn setup_codegen<'ctx>(context: &'ctx Context) -> codegen_ast::ASTCodeGen<'ctx> {
        codegen_ast::ASTCodeGen::new(context, "test_module")
    }

    /// Test: build_mangled_method_name consistency
    /// Ensures that method name mangling is deterministic for generic types
    #[test]
    fn test_build_mangled_method_name_simple() {
        let context = Context::create();
        let mut codegen = setup_codegen(&context);

        // Register Vec<i32> struct
        let struct_name = "Vec";
        let type_args = vec![Type::I32];
        let method_name = "push";

        // Expected: Vec_i32_push
        let mangled = format!(
            "{}_{}_{}",
            struct_name,
            codegen.type_to_string(&type_args[0]),
            method_name
        );

        assert_eq!(mangled, "Vec_i32_push");
    }

    #[test]
    fn test_build_mangled_method_name_multi_param() {
        let context = Context::create();
        let mut codegen = setup_codegen(&context);

        // HashMap<String, i32>
        let struct_name = "HashMap";
        let type_args = vec![Type::String, Type::I32];
        let method_name = "insert";

        let type_names: Vec<String> = type_args
            .iter()
            .map(|t| codegen.type_to_string(t))
            .collect();

        let mangled = format!("{}_{}_{}", struct_name, type_names.join("_"), method_name);

        assert_eq!(mangled, "HashMap_string_i32_insert");
    }

    /// Test: Receiver type from variable_concrete_types
    /// Method instantiation should use tracked variable type, not re-infer
    #[test]
    fn test_receiver_type_from_variable_concrete_types() {
        let context = Context::create();
        let mut codegen = setup_codegen(&context);

        // Register variable with concrete type
        let var_name = "v";
        let var_type = Type::Generic {
            name: "Vec".to_string(),
            type_args: vec![Type::I32],
        };

        codegen
            .variable_concrete_types
            .insert(var_name.to_string(), var_type.clone());

        // Retrieve type
        let retrieved = codegen.variable_concrete_types.get(var_name).unwrap();

        match retrieved {
            Type::Generic { name, type_args } => {
                assert_eq!(name, "Vec");
                assert_eq!(type_args[0], Type::I32);
            }
            _ => panic!("Expected Generic type"),
        }
    }

    /// Test: Type args extraction from Generic type
    #[test]
    fn test_extract_type_args_from_generic() {
        let context = Context::create();
        let codegen = setup_codegen(&context);

        let ty = Type::Generic {
            name: "Vec".to_string(),
            type_args: vec![Type::I32],
        };

        let extracted = codegen.extract_type_args_from_type(&ty).unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0], Type::I32);
    }

    /// Test: Mangled name consistency between struct and method
    /// Vec<i32> variable should generate Vec_i32 struct and Vec_i32_push method
    #[test]
    fn test_mangled_name_consistency() {
        let context = Context::create();
        let mut codegen = setup_codegen(&context);

        let struct_name = "Vec";
        let type_args = vec![Type::I32];

        // Struct mangled name
        let struct_mangled = format!("{}_{}", struct_name, codegen.type_to_string(&type_args[0]));

        // Method mangled name (should use same struct prefix)
        let method_mangled = format!(
            "{}_{}_{}",
            struct_name,
            codegen.type_to_string(&type_args[0]),
            "push"
        );

        assert_eq!(struct_mangled, "Vec_i32");
        assert_eq!(method_mangled, "Vec_i32_push");
        assert!(method_mangled.starts_with(&struct_mangled));
    }

    /// Test: type_to_string normalization
    /// Ensures consistent type name generation for mangling
    #[test]
    fn test_type_to_string_normalization() {
        let context = Context::create();
        let codegen = setup_codegen(&context);

        // Primitive types
        assert_eq!(codegen.type_to_string(&Type::I32), "i32");
        assert_eq!(codegen.type_to_string(&Type::I64), "i64");
        assert_eq!(codegen.type_to_string(&Type::F64), "f64");
        assert_eq!(codegen.type_to_string(&Type::Bool), "bool");
        assert_eq!(codegen.type_to_string(&Type::String), "string");

        // Generic types
        let vec_i32 = Type::Generic {
            name: "Vec".to_string(),
            type_args: vec![Type::I32],
        };
        assert_eq!(codegen.type_to_string(&vec_i32), "Vec_i32");

        // Nested generics
        let vec_vec_i32 = Type::Generic {
            name: "Vec".to_string(),
            type_args: vec![vec_i32.clone()],
        };
        assert_eq!(codegen.type_to_string(&vec_vec_i32), "Vec_Vec_i32");
    }
}
