// Phase 1: Type Inference Foundation - Unit Tests
// Test variable type tracking, context-aware inference, and Unknown type handling

use vex_ast::*;
use vex_compiler::codegen_ast;

#[cfg(test)]
mod phase1_type_inference_tests {
    use super::*;
    use inkwell::context::Context;

    fn setup_codegen<'ctx>(context: &'ctx Context) -> codegen_ast::ASTCodeGen<'ctx> {
        codegen_ast::ASTCodeGen::new(context, "test_module")
    }

    #[test]
    fn test_unknown_type_creation() {
        // Type::Unknown should be created for placeholders
        let unknown = Type::Unknown;
        assert_eq!(format!("{:?}", unknown), "Unknown");
    }

    #[test]
    fn test_variable_concrete_types_tracking() {
        let context = Context::create();
        let mut codegen = setup_codegen(&context);

        // Register a variable with concrete type
        codegen.variable_concrete_types.insert(
            "v".to_string(),
            Type::Generic {
                name: "Vec".to_string(),
                type_args: vec![Type::I32],
            },
        );

        // Retrieve and verify
        let v_type = codegen.variable_concrete_types.get("v").unwrap();
        match v_type {
            Type::Generic { name, type_args } => {
                assert_eq!(name, "Vec");
                assert_eq!(type_args.len(), 1);
                assert_eq!(type_args[0], Type::I32);
            }
            _ => panic!("Expected Generic type"),
        }
    }

    #[test]
    fn test_type_constraint_creation() {
        use codegen_ast::TypeConstraint;

        // Equal constraint
        let c1 = TypeConstraint::Equal(Type::Unknown, Type::I32);
        assert!(matches!(c1, TypeConstraint::Equal(_, _)));

        // MethodReceiver constraint
        let c2 = TypeConstraint::MethodReceiver {
            receiver_name: "v".to_string(),
            method_name: "push".to_string(),
            arg_types: vec![Type::I32],
            inferred_receiver_type: Type::Generic {
                name: "Vec".to_string(),
                type_args: vec![Type::I32],
            },
        };
        assert!(matches!(c2, TypeConstraint::MethodReceiver { .. }));

        // Assignment constraint
        let c3 = TypeConstraint::Assignment {
            var_name: "x".to_string(),
            expr_type: Type::F64,
        };
        assert!(matches!(c3, TypeConstraint::Assignment { .. }));
    }

    #[test]
    fn test_unify_types_simple() {
        let context = Context::create();
        let mut codegen = setup_codegen(&context);

        // Variable with Unknown
        codegen
            .variable_concrete_types
            .insert("x".to_string(), Type::Unknown);

        // Constraint: Unknown = i32
        codegen
            .type_constraints
            .push(codegen_ast::TypeConstraint::Equal(Type::Unknown, Type::I32));

        // Unify
        codegen.unify_types().unwrap();

        // Check that x is now i32
        let x_type = codegen.variable_concrete_types.get("x").unwrap();
        assert_eq!(*x_type, Type::I32);
    }

    #[test]
    fn test_unify_types_generic() {
        let context = Context::create();
        let mut codegen = setup_codegen(&context);

        // Variable: Vec<Unknown>
        codegen.variable_concrete_types.insert(
            "v".to_string(),
            Type::Generic {
                name: "Vec".to_string(),
                type_args: vec![Type::Unknown],
            },
        );

        // Constraint: v has type Vec<i32> from method call
        codegen
            .type_constraints
            .push(codegen_ast::TypeConstraint::MethodReceiver {
                receiver_name: "v".to_string(),
                method_name: "push".to_string(),
                arg_types: vec![Type::I32],
                inferred_receiver_type: Type::Generic {
                    name: "Vec".to_string(),
                    type_args: vec![Type::I32],
                },
            });

        // Unify
        codegen.unify_types().unwrap();

        // Check that v is now Vec<i32>
        let v_type = codegen.variable_concrete_types.get("v").unwrap();
        match v_type {
            Type::Generic { name, type_args } => {
                assert_eq!(name, "Vec");
                assert_eq!(type_args.len(), 1);
                assert_eq!(type_args[0], Type::I32);
            }
            _ => panic!("Expected Generic type"),
        }
    }

    #[test]
    fn test_unify_types_fails_on_unresolved() {
        let context = Context::create();
        let mut codegen = setup_codegen(&context);

        // Variable with Unknown
        codegen
            .variable_concrete_types
            .insert("y".to_string(), Type::Unknown);

        // No constraints - should fail
        let result = codegen.unify_types();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot infer complete type"));
    }
}
