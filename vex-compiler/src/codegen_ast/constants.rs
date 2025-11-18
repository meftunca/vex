// Constant compilation for Vex
use super::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile a constant declaration
    /// Constants are global immutable values
    pub fn compile_const(&mut self, const_decl: &Const) -> Result<(), String> {
        // Evaluate the constant expression at compile time
        let value = self.compile_expression(&const_decl.value)?;

        // Determine the type
        let llvm_type = if let Some(ref ty) = const_decl.ty {
            self.ast_type_to_llvm(ty)
        } else {
            // Infer type from value
            value.get_type()
        };

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

        Ok(())
    }
}
