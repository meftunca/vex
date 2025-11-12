//! Built-in functions for Vex channels (MPSC queue).

use inkwell::module::Linkage;
use inkwell::types::{PointerType, StructType};
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

use crate::codegen_ast::ASTCodeGen;

// Opaque struct type for the channel handle
const CHANNEL_TYPE_NAME: &str = "struct.VexChannel";

// External C function names (match vex_channel.h)
const CREATE_FN: &str = "vex_channel_create";
const SEND_FN: &str = "vex_channel_send";
const RECV_FN: &str = "vex_channel_recv";

/// Builtin: Channel.new<T>(capacity: i64) -> Channel<T>
pub fn builtin_channel_new<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[inkwell::values::BasicValueEnum<'ctx>],
) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "Channel.new expects 1 argument (capacity), found {}",
            args.len()
        ));
    }
    let capacity = args[0].into_int_value();
    let create_fn = codegen.get_or_declare_vex_channel_create();
    let call_site_value = codegen
        .builder
        .build_call(create_fn, &[capacity.into()], "new_channel")
        .map_err(|e| e.to_string())?;

    Ok(call_site_value.try_as_basic_value().left().unwrap())
}

/// Builtin: Channel.send(channel: Channel<T>, value: T)
pub(super) fn builtin_channel_send<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[inkwell::values::BasicValueEnum<'ctx>],
) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!(
            "Channel.send expects 2 arguments (channel, value), found {}",
            args.len()
        ));
    }
    let channel_ptr = args[0].into_pointer_value();
    let value_ptr = codegen
        .builder
        .build_alloca(args[1].get_type(), "send_val_alloca")
        .map_err(|e| e.to_string())?;
    codegen
        .builder
        .build_store(value_ptr, args[1])
        .map_err(|e| e.to_string())?;

    // Cast value_ptr to void* (i8*) for vex_channel_send
    let void_ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let value_as_void_ptr = codegen
        .builder
        .build_pointer_cast(value_ptr, void_ptr_type, "value_as_void_ptr")
        .map_err(|e| e.to_string())?;

    let send_fn = codegen.get_or_declare_vex_channel_send();
    let status = codegen
        .builder
        .build_call(
            send_fn,
            &[channel_ptr.into(), value_as_void_ptr.into()],
            "send_status",
        )
        .map_err(|e| e.to_string())?
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

    Ok(status.into())
}

/// Builtin: Channel.recv(channel: Channel<T>) -> T
pub(super) fn builtin_channel_recv<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[inkwell::values::BasicValueEnum<'ctx>],
) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "Channel.recv expects 1 argument (channel), found {}",
            args.len()
        ));
    }
    let channel_ptr = args[0].into_pointer_value();
    let i8_ptr_ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let recv_val_alloca = codegen
        .builder
        .build_alloca(i8_ptr_ptr_type, "recv_val_alloca")
        .map_err(|e| e.to_string())?;

    let recv_fn = codegen.get_or_declare_vex_channel_recv();
    let _status = codegen
        .builder
        .build_call(
            recv_fn,
            &[channel_ptr.into(), recv_val_alloca.into()],
            "recv_status",
        )
        .map_err(|e| e.to_string())?
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_int_value();

    let received_ptr = codegen
        .builder
        .build_load(i8_ptr_ptr_type, recv_val_alloca, "loaded_recv_ptr")
        .map_err(|e| e.to_string())?
        .into_pointer_value();

    // For now, let's assume we are dealing with simple types and just return the pointer.
    // A full implementation would need to know the type `T` to load correctly.
    // This is a placeholder.
    let i64_val = codegen
        .builder
        .build_ptr_to_int(received_ptr, codegen.context.i64_type(), "ptr_to_int")
        .map_err(|e| e.to_string())?;

    Ok(i64_val.into())
}

impl<'ctx> ASTCodeGen<'ctx> {
    // --- Function Declarations ---

    pub(super) fn get_or_declare_vex_channel_create(&self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function(CREATE_FN) {
            return func;
        }
        let channel_ptr_type = self.get_channel_ptr_type();
        let i64_type = self.context.i64_type();
        let fn_type = channel_ptr_type.fn_type(&[i64_type.into()], false);
        self.module
            .add_function(CREATE_FN, fn_type, Some(Linkage::External))
    }

    pub fn get_or_declare_vex_channel_send(&self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function(SEND_FN) {
            return func;
        }
        let channel_ptr_type = self.get_channel_ptr_type();
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let i32_type = self.context.i32_type(); // For status enum
        let fn_type = i32_type.fn_type(&[channel_ptr_type.into(), i8_ptr_type.into()], false);
        self.module
            .add_function(SEND_FN, fn_type, Some(Linkage::External))
    }

    pub fn get_or_declare_vex_channel_recv(&self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function(RECV_FN) {
            return func;
        }
        // C signature: vex_channel_status_t vex_channel_recv(vex_channel_t* chan, void** data_out)
        let channel_ptr_type = self.get_channel_ptr_type();
        let void_ptr_ptr_type = self.context.ptr_type(AddressSpace::default()); // void**
        let i32_type = self.context.i32_type(); // vex_channel_status_t (enum)
        let fn_type = i32_type.fn_type(&[channel_ptr_type.into(), void_ptr_ptr_type.into()], false);
        self.module
            .add_function(RECV_FN, fn_type, Some(Linkage::External))
    }

    // --- Type Helpers ---

    pub(super) fn get_channel_ptr_type(&self) -> PointerType<'ctx> {
        self.get_or_define_channel_type();
        self.context.ptr_type(AddressSpace::default())
    }

    fn get_or_define_channel_type(&self) -> StructType<'ctx> {
        if let Some(struct_type) = self.module.get_struct_type(CHANNEL_TYPE_NAME) {
            return struct_type;
        }
        self.context.opaque_struct_type(CHANNEL_TYPE_NAME)
    }
}
