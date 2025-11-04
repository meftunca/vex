// Function declaration: declare function signatures without bodies

use super::ASTCodeGen;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Declare a function signature (without body)
    pub(crate) fn declare_function(
        &mut self,
        func: &Function,
    ) -> Result<FunctionValue<'ctx>, String> {
        // Determine the function name (with mangling for methods)
        let fn_name = if let Some(ref receiver) = func.receiver {
            // Extract type name from receiver
            let type_name = match &receiver.ty {
                Type::Named(name) => name.clone(),
                Type::Reference(inner, _) => {
                    if let Type::Named(name) = &**inner {
                        name.clone()
                    } else {
                        return Err(
                            "Receiver must be a named type or reference to named type".to_string()
                        );
                    }
                }
                _ => {
                    return Err(
                        "Receiver must be a named type or reference to named type".to_string()
                    );
                }
            };

            // Mangle method name: TypeName_methodName
            format!("{}_{}", type_name, func.name)
        } else {
            func.name.clone()
        };

        // Build parameter types (receiver becomes first parameter if present)
        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();

        if let Some(ref receiver) = func.receiver {
            param_types.push(self.ast_type_to_llvm(&receiver.ty).into());
        }

        for param in &func.params {
            param_types.push(self.ast_type_to_llvm(&param.ty).into());
        }

        // Build return type
        let ret_type = if let Some(ref ty) = func.return_type {
            let llvm_ret = self.ast_type_to_llvm(ty);
            eprintln!(
                "🟢 Function {} return type: {:?} → LLVM: {:?}",
                fn_name, ty, llvm_ret
            );
            llvm_ret
        } else {
            BasicTypeEnum::IntType(self.context.i32_type()) // Default to i32
        };

        // Create function type
        let fn_type = match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            _ => {
                return Err(format!(
                    "Unsupported return type for function {}",
                    func.name
                ));
            }
        };

        // Add function to module (use mangled name)
        let fn_val = self.module.add_function(&fn_name, fn_type, None);
        self.functions.insert(fn_name.clone(), fn_val);

        Ok(fn_val)
    }

    /// Declare a trait impl method with proper name mangling
    pub(crate) fn declare_trait_impl_method(
        &mut self,
        trait_name: &str,
        for_type: &Type,
        method: &Function,
    ) -> Result<(), String> {
        let type_name = match for_type {
            Type::Named(name) => name,
            _ => return Err(format!("Expected named type, got: {:?}", for_type)),
        };

        // Mangle name: TypeName_TraitName_methodName
        // Example: Point_Printable_print
        let mangled_name = format!("{}_{}_{}", type_name, trait_name, method.name);

        // Build parameter types (receiver becomes first parameter)
        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();

        if let Some(ref receiver) = method.receiver {
            param_types.push(self.ast_type_to_llvm(&receiver.ty).into());
        }

        for param in &method.params {
            param_types.push(self.ast_type_to_llvm(&param.ty).into());
        }

        // Build return type
        let ret_type = if let Some(ref ty) = method.return_type {
            self.ast_type_to_llvm(ty)
        } else {
            inkwell::types::BasicTypeEnum::IntType(self.context.i32_type())
        };

        // Create function type and declare function
        use inkwell::types::BasicTypeEnum;
        let fn_type = match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ScalableVectorType(t) => t.fn_type(&param_types, false),
        };

        let fn_val = self.module.add_function(&mangled_name, fn_type, None);
        self.functions.insert(mangled_name.clone(), fn_val);

        // Store function def for later compilation
        let mut mangled_method = method.clone();
        mangled_method.name = mangled_name.clone();
        self.function_defs.insert(mangled_name, mangled_method);

        Ok(())
    }

    /// Declare an inline struct method (new trait system v1.3)
    /// Inline methods are declared directly inside struct body: struct Foo { ... fn bar() {...} }
    pub(crate) fn declare_struct_method(
        &mut self,
        struct_name: &str,
        method: &Function,
    ) -> Result<(), String> {
        // Mangle name: StructName_methodName
        // Example: FileLogger_log
        let mangled_name = format!("{}_{}", struct_name, method.name);

        // Build parameter types (receiver becomes first parameter)
        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();

        if let Some(ref receiver) = method.receiver {
            param_types.push(self.ast_type_to_llvm(&receiver.ty).into());
        }

        for param in &method.params {
            param_types.push(self.ast_type_to_llvm(&param.ty).into());
        }

        // Build return type
        let ret_type = if let Some(ref ty) = method.return_type {
            self.ast_type_to_llvm(ty)
        } else {
            inkwell::types::BasicTypeEnum::IntType(self.context.i32_type())
        };

        // Create function type and declare function
        use inkwell::types::BasicTypeEnum;
        let fn_type = match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ScalableVectorType(t) => t.fn_type(&param_types, false),
        };

        let fn_val = self.module.add_function(&mangled_name, fn_type, None);
        self.functions.insert(mangled_name.clone(), fn_val);

        // Store function def for later compilation
        let mut mangled_method = method.clone();
        mangled_method.name = mangled_name.clone();
        self.function_defs.insert(mangled_name, mangled_method);

        Ok(())
    }

    /// Generate constructor functions for enum variants
    /// For C-style enums: Color::Red -> Color_Red() returns i32 (tag value)
    /// For data-carrying enums: Option::Some(T) -> Option_Some(value: T) returns struct
    pub(crate) fn generate_enum_constructors(&mut self, enum_def: &Enum) -> Result<(), String> {
        // Data-carrying enums are represented as structs with two fields:
        // - tag: i32 (variant discriminant)
        // - data: union of all variant data types
        // For simplicity, we'll use the largest data type and cast as needed

        for (tag_index, variant) in enum_def.variants.iter().enumerate() {
            let constructor_name = format!("{}_{}", enum_def.name, variant.name);

            if let Some(ref data_type) = variant.data {
                // Data-carrying variant: create constructor function that takes the data
                // and returns a struct {tag: i32, data: T}

                let data_llvm_type = self.ast_type_to_llvm(data_type);

                // Get or create enum struct type: {i32, T}
                let i32_type = self.context.i32_type();
                let enum_struct_type = self
                    .context
                    .struct_type(&[i32_type.into(), data_llvm_type], false);

                // Constructor function: fn(data: T) -> {i32, T}
                let fn_type = enum_struct_type.fn_type(&[data_llvm_type.into()], false);
                let function = self.module.add_function(&constructor_name, fn_type, None);

                // Create function body
                let entry = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(entry);

                // Get data parameter
                let data_param = function
                    .get_nth_param(0)
                    .ok_or_else(|| "Missing data parameter".to_string())?;

                // Create enum struct value
                let undef_struct = enum_struct_type.get_undef();

                // Insert tag value at index 0
                let tag_value = i32_type.const_int(tag_index as u64, false);
                let with_tag = self
                    .builder
                    .build_insert_value(undef_struct, tag_value, 0, "with_tag")
                    .map_err(|e| format!("Failed to insert tag: {}", e))?;

                // Insert data value at index 1
                let enum_value = self
                    .builder
                    .build_insert_value(with_tag, data_param, 1, "enum_value")
                    .map_err(|e| format!("Failed to insert data: {}", e))?;

                // Convert AggregateValueEnum to BasicValueEnum
                let enum_basic_value: BasicValueEnum = match enum_value {
                    inkwell::values::AggregateValueEnum::ArrayValue(v) => v.into(),
                    inkwell::values::AggregateValueEnum::StructValue(v) => v.into(),
                };

                // Return the constructed enum value
                self.builder
                    .build_return(Some(&enum_basic_value))
                    .map_err(|e| format!("Failed to build return: {}", e))?;

                // Store function for later use
                self.functions.insert(constructor_name, function);
            } else {
                // Unit variant: just return tag value (i32)
                let i32_type = self.context.i32_type();
                let fn_type = i32_type.fn_type(&[], false);
                let function = self.module.add_function(&constructor_name, fn_type, None);

                // Create function body
                let entry = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(entry);

                // Return tag value
                let tag_value = i32_type.const_int(tag_index as u64, false);
                self.builder
                    .build_return(Some(&tag_value))
                    .map_err(|e| format!("Failed to build return: {}", e))?;

                // Store function for later use
                self.functions.insert(constructor_name, function);
            }
        }

        Ok(())
    }
}


