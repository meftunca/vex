// Async runtime builtins for Vex
// Wraps vex_async_io C runtime

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// runtime_create(num_workers: i32) -> ptr
    pub(crate) fn builtin_runtime_create(
        &mut self,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if args.len() != 1 {
            return Err("runtime_create expects 1 argument (num_workers)".to_string());
        }

        let num_workers = args[0].into_int_value();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        let runtime_create_fn = self.declare_runtime_fn(
            "runtime_create",
            &[self.context.i32_type().into()],
            ptr_type.into(),
        );

        let result = self
            .builder
            .build_call(runtime_create_fn, &[num_workers.into()], "runtime_create")
            .map_err(|e| format!("Failed to call runtime_create: {}", e))?
            .try_as_basic_value()
            .unwrap_basic();

        Ok(result)
    }

    /// runtime_destroy(runtime: ptr)
    pub(crate) fn builtin_runtime_destroy(
        &mut self,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if args.len() != 1 {
            return Err("runtime_destroy expects 1 argument (runtime)".to_string());
        }

        let runtime_ptr = args[0].into_pointer_value();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        let runtime_destroy_fn = self.declare_runtime_fn(
            "runtime_destroy",
            &[ptr_type.into()],
            self.context.i32_type().into(), // Return i32 instead of void
        );

        self.builder
            .build_call(runtime_destroy_fn, &[runtime_ptr.into()], "runtime_destroy")
            .map_err(|e| format!("Failed to call runtime_destroy: {}", e))?;

        // Return 0
        Ok(self.context.i32_type().const_int(0, false).into())
    }

    /// runtime_run(runtime: ptr)
    pub(crate) fn builtin_runtime_run(
        &mut self,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if args.len() != 1 {
            return Err("runtime_run expects 1 argument (runtime)".to_string());
        }

        let runtime_ptr = args[0].into_pointer_value();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        let runtime_run_fn = self.declare_runtime_fn(
            "runtime_run",
            &[ptr_type.into()],
            self.context.i32_type().into(),
        );

        self.builder
            .build_call(runtime_run_fn, &[runtime_ptr.into()], "runtime_run")
            .map_err(|e| format!("Failed to call runtime_run: {}", e))?;

        // Return void (nil)
        Ok(self.context.i32_type().const_int(0, false).into())
    }

    /// runtime_shutdown(runtime: ptr)
    pub(crate) fn builtin_runtime_shutdown(
        &mut self,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if args.len() != 1 {
            return Err("runtime_shutdown expects 1 argument (runtime)".to_string());
        }

        let runtime_ptr = args[0].into_pointer_value();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        let runtime_shutdown_fn = self.declare_runtime_fn(
            "runtime_shutdown",
            &[ptr_type.into()],
            self.context.i32_type().into(),
        );

        self.builder
            .build_call(
                runtime_shutdown_fn,
                &[runtime_ptr.into()],
                "runtime_shutdown",
            )
            .map_err(|e| format!("Failed to call runtime_shutdown: {}", e))?;

        // Return void (nil)
        Ok(self.context.i32_type().const_int(0, false).into())
    }

    /// async_sleep(millis: i64)
    /// Simple async sleep using worker_await_after
    pub(crate) fn builtin_async_sleep(
        &mut self,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if args.len() != 1 {
            return Err("async_sleep expects 1 argument (millis)".to_string());
        }

        let millis = args[0].into_int_value();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        // For now, we'll just call a C function vex_async_sleep
        // which internally uses worker_await_after
        let async_sleep_fn = self.declare_runtime_fn(
            "vex_async_sleep",
            &[self.context.i64_type().into()],
            self.context.i32_type().into(),
        );

        self.builder
            .build_call(async_sleep_fn, &[millis.into()], "async_sleep")
            .map_err(|e| format!("Failed to call async_sleep: {}", e))?;

        // Return void (nil)
        Ok(self.context.i32_type().const_int(0, false).into())
    }

    /// spawn_async(fn_ptr: ptr, args: ptr) -> ptr
    /// Spawns an async task and returns a handle
    pub(crate) fn builtin_spawn_async(
        &mut self,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if args.len() != 2 {
            return Err("spawn_async expects 2 arguments (fn_ptr, args)".to_string());
        }

        let fn_ptr = args[0].into_pointer_value();
        let args_ptr = args[1].into_pointer_value();
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        let spawn_async_fn = self.declare_runtime_fn(
            "vex_spawn_async",
            &[ptr_type.into(), ptr_type.into()],
            ptr_type.into(),
        );

        let result = self
            .builder
            .build_call(
                spawn_async_fn,
                &[fn_ptr.into(), args_ptr.into()],
                "spawn_async",
            )
            .map_err(|e| format!("Failed to call spawn_async: {}", e))?
            .try_as_basic_value()
            .unwrap_basic();

        Ok(result)
    }
}
