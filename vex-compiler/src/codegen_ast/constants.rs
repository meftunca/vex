// Constant compilation for Vex
use super::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile a constant declaration
    /// Constants are global immutable values
    pub fn compile_const(&mut self, const_decl: &Const) -> Result<(), String> {
        eprintln!(
            "🔨 compile_const: {} type={:?}",
            const_decl.name, const_decl.ty
        );
        // Evaluate the constant expression with expected type for target-typed inference
        let value = if let Some(ref ty) = const_decl.ty {
            self.compile_expression_with_type(&const_decl.value, Some(ty))?
        } else {
            self.compile_expression(&const_decl.value)?
        };

        eprintln!(
            "   Result value type: {:?} (expected: {:?})",
            value.get_type(),
            const_decl.ty
        );
        if let BasicValueEnum::IntValue(int_val) = value {
            if let Some(const_val) = int_val.get_sign_extended_constant() {
                eprintln!("   Const value: {}", const_val);
            } else {
                eprintln!("   ⚠️ Value is not a compile-time constant!");
            }
        }

        // ⚠️ CRITICAL FIX: Use value's actual type, not ast_type_to_llvm
        // The value already has the correct LLVM type from compile_expression
        let llvm_type = value.get_type();

        // Create a global constant
        let global = self.module.add_global(llvm_type, None, &const_decl.name);
        global.set_initializer(&value);
        global.set_constant(true); // Mark as constant (immutable)
        global.set_linkage(inkwell::module::Linkage::Internal); // Internal linkage

        // Store in GLOBAL constants map (never cleared during function compilation)
        self.global_constants
            .insert(const_decl.name.clone(), global.as_pointer_value());
        self.global_constant_types
            .insert(const_decl.name.clone(), llvm_type);

        // ⭐ NEW: Also store constant value for namespace access (math.PI)
        // This allows field_access to resolve math.PI without loading from global
        self.module_constants.insert(const_decl.name.clone(), value);

        // ⭐ NEW: Store AST type for proper type inference in println() etc.
        if let Some(ref ty) = const_decl.ty {
            self.module_constant_types
                .insert(const_decl.name.clone(), ty.clone());
        }

        Ok(())
    }
}
