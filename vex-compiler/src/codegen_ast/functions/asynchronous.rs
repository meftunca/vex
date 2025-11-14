// src/codegen/functions/asynchronous.rs
use super::super::*;
use super::await_scanner::count_await_points;
use inkwell::values::FunctionValue;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn compile_async_function(&mut self, func: &Function) -> Result<(), String> {
        // Async functions are transformed into state machines:
        // 1. Create a state struct with all locals + state field
        // 2. Generate a resume function: CoroStatus resume_fn(WorkerContext*, void* state)
        // 3. At await points, save state and return CORO_STATUS_YIELDED
        // 4. Original function becomes a spawn wrapper

        let fn_name = &func.name;

        // Step 1: Generate state struct type
        let state_struct_name = format!("{}_AsyncState", fn_name);
        let mut state_fields = vec![];

        // Add state field (i32 for state machine)
        state_fields.push(self.context.i32_type().into());

        // Add fields for all parameters
        for param in &func.params {
            state_fields.push(self.ast_type_to_llvm(&param.ty));
        }

        // Add fields for all local variables (collect from body)
        // For now, we'll allocate them dynamically as needed

        let state_struct_type = self.context.struct_type(&state_fields, false);

        // Step 2: Generate resume function
        // CoroStatus (*coro_resume_func)(WorkerContext* context, void* coro_data);
        let worker_ctx_ptr = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let void_ptr = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let coro_status_type = self.context.i32_type(); // enum CoroStatus

        let resume_fn_type =
            coro_status_type.fn_type(&[worker_ctx_ptr.into(), void_ptr.into()], false);
        let resume_fn_name = format!("{}_resume", fn_name);
        let resume_fn = self
            .module
            .add_function(&resume_fn_name, resume_fn_type, None);

        // Build resume function body
        let entry = self.context.append_basic_block(resume_fn, "entry");
        self.builder.position_at_end(entry);

        let ctx_param = resume_fn
            .get_nth_param(0)
            .ok_or_else(|| {
                format!(
                    "Failed to get context parameter for async function {}",
                    fn_name
                )
            })?
            .into_pointer_value();
        let state_param = resume_fn
            .get_nth_param(1)
            .ok_or_else(|| {
                format!(
                    "Failed to get state parameter for async function {}",
                    fn_name
                )
            })?
            .into_pointer_value();

        // Cast state_param to state struct
        let state_ptr = self
            .builder
            .build_pointer_cast(
                state_param,
                state_struct_type.ptr_type(inkwell::AddressSpace::default()),
                "state_cast",
            )
            .map_err(|e| format!("Failed to cast state: {}", e))?;

        // Load state field
        let state_field_ptr = self
            .builder
            .build_struct_gep(state_struct_type, state_ptr, 0, "state_field_ptr")
            .map_err(|e| format!("Failed to get state field: {}", e))?;

        let current_state = self
            .builder
            .build_load(self.context.i32_type(), state_field_ptr, "current_state")
            .map_err(|e| format!("Failed to load state: {}", e))?;

        // ⭐ PRE-SCAN: Count total await points in function body
        let await_count = count_await_points(&func.body);

        // Create switch on state - pre-allocate ALL blocks before building switch
        let state0_block = self.context.append_basic_block(resume_fn, "state0");
        let done_block = self.context.append_basic_block(resume_fn, "done");

        // Pre-create resume blocks for each await point
        let mut resume_blocks = vec![];
        for i in 1..=await_count {
            let resume_block = self
                .context
                .append_basic_block(resume_fn, &format!("resume_{}", i));
            resume_blocks.push(resume_block);
        }

        // Build switch with ALL cases (state 0 + all resume states)
        let mut switch_cases = vec![(self.context.i32_type().const_int(0, false), state0_block)];
        for (i, block) in resume_blocks.iter().enumerate() {
            let state_id = (i + 1) as u64;
            switch_cases.push((self.context.i32_type().const_int(state_id, false), *block));
        }

        self.builder
            .build_switch(current_state.into_int_value(), done_block, &switch_cases)
            .map_err(|e| format!("Failed to build switch: {}", e))?;

        // State 0: Initial execution
        self.builder.position_at_end(state0_block);

        // Load parameters from state struct
        self.current_function = Some(resume_fn);
        self.current_async_resume_fn = Some(resume_fn);
        self.variables.clear();
        self.variable_types.clear();

        // Push state machine context for await expressions
        // Store resume blocks in codegen state for await compilation
        self.async_resume_blocks = resume_blocks.clone();
        self.async_state_stack.push((state_ptr, state_field_ptr, 0));
        eprintln!(
            "🔧 Pushed state context: state_id=0, resume_blocks_count={}",
            resume_blocks.len()
        );

        for (i, param) in func.params.iter().enumerate() {
            let param_ptr = self
                .builder
                .build_struct_gep(
                    state_struct_type,
                    state_ptr,
                    (i + 1) as u32, // +1 because state field is at 0
                    &format!("param_{}_ptr", param.name),
                )
                .map_err(|e| format!("Failed to get param ptr: {}", e))?;

            let param_type = self.ast_type_to_llvm(&param.ty);
            self.variables.insert(param.name.clone(), param_ptr);
            self.variable_types.insert(param.name.clone(), param_type);

            // ⭐ CRITICAL: Store AST type for type inference
            self.variable_ast_types
                .insert(param.name.clone(), param.ty.clone());
        }

        // Compile function body (await expressions will use state machine context)
        self.compile_block(&func.body)?;

        // Pop state machine context and clear resume blocks
        self.async_state_stack.pop();
        self.current_async_resume_fn = None;

        // ⭐ Resume blocks already have terminators (added during await compilation)
        // Just verify and add fallback if needed
        let done_status = self.context.i32_type().const_int(2, false);

        // ⚠️ CRITICAL: Position builder back to state0 block's end before checking terminators
        let current_insert_block = self.builder.get_insert_block();

        for resume_block in &self.async_resume_blocks {
            // Each resume block should already have a branch to continuation
            // If it doesn't, add DONE return as fallback
            if resume_block.get_terminator().is_none() {
                self.builder.position_at_end(*resume_block);
                self.builder.build_return(Some(&done_status)).map_err(|e| {
                    format!("Failed to build resume block fallback terminator: {}", e)
                })?;
            }
        }

        // Restore builder position
        if let Some(block) = current_insert_block {
            self.builder.position_at_end(block);
        }

        // Clear resume blocks for next async function
        self.async_resume_blocks.clear();

        // ⚠️ CRITICAL: Only add return if block doesn't already have terminator
        // (compile_return_statement already adds one)
        let current_block = self
            .builder
            .get_insert_block()
            .ok_or_else(|| format!("No current block in async function {}", fn_name))?;
        if current_block.get_terminator().is_none() {
            // Return CORO_STATUS_DONE (2)
            self.builder
                .build_return(Some(&done_status))
                .map_err(|e| format!("Failed to build return: {}", e))?;
        }

        // Done block: already finished
        self.builder.position_at_end(done_block);
        self.builder
            .build_return(Some(&done_status))
            .map_err(|e| format!("Failed to build return: {}", e))?;

        // Step 3: Generate wrapper function (original name)
        // This allocates state and spawns the coroutine
        let fn_val = *self
            .functions
            .get(fn_name)
            .ok_or_else(|| format!("Async function {} not declared", fn_name))?;

        self.current_function = Some(fn_val);
        let wrapper_entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(wrapper_entry);

        // Allocate state struct
        let malloc_fn = self.get_or_declare_malloc();
        let state_size = state_struct_type.size_of().ok_or_else(|| {
            format!(
                "Failed to get size of state struct for async function {}",
                fn_name
            )
        })?;
        let state_alloc = self
            .builder
            .build_call(malloc_fn, &[state_size.into()], "state_alloc")
            .map_err(|e| format!("Failed to call malloc: {}", e))?;

        let state_alloc_ptr = state_alloc
            .try_as_basic_value()
            .unwrap_basic()
            .into_pointer_value();

        let state_alloc_typed = self
            .builder
            .build_pointer_cast(
                state_alloc_ptr,
                state_struct_type.ptr_type(inkwell::AddressSpace::default()),
                "state_typed",
            )
            .map_err(|e| format!("Failed to cast: {}", e))?;

        // Initialize state field to 0
        let state_init_ptr = self
            .builder
            .build_struct_gep(state_struct_type, state_alloc_typed, 0, "state_init_ptr")
            .map_err(|e| format!("Failed to get state ptr: {}", e))?;

        self.builder
            .build_store(state_init_ptr, self.context.i32_type().const_int(0, false))
            .map_err(|e| format!("Failed to store state: {}", e))?;

        // Copy parameters into state struct
        for (i, param) in func.params.iter().enumerate() {
            let param_val = fn_val.get_nth_param(i as u32).ok_or_else(|| {
                format!(
                    "Failed to get parameter {} for async function {}",
                    i, fn_name
                )
            })?;
            let param_dest = self
                .builder
                .build_struct_gep(
                    state_struct_type,
                    state_alloc_typed,
                    (i + 1) as u32,
                    &format!("param_{}_dest", i),
                )
                .map_err(|e| format!("Failed to get param dest: {}", e))?;

            self.builder
                .build_store(param_dest, param_val)
                .map_err(|e| format!("Failed to store param: {}", e))?;
        }

        // Spawn coroutine: runtime_spawn_global(runtime, resume_fn, state)
        let spawn_fn = self.get_or_declare_runtime_spawn();

        // Load global runtime handle: Runtime* rt = __vex_global_runtime;
        let global_runtime_var = self
            .global_runtime
            .ok_or("No global runtime for async function - async functions require runtime initialization in main")?;

        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let runtime_handle = self
            .builder
            .build_load(ptr_type, global_runtime_var, "runtime_load")
            .map_err(|e| format!("Failed to load runtime: {}", e))?
            .into_pointer_value();

        // Cast resume_fn to void*
        let resume_fn_ptr = resume_fn.as_global_value().as_pointer_value();
        let resume_void_ptr = self
            .builder
            .build_pointer_cast(
                resume_fn_ptr,
                self.context
                    .i8_type()
                    .ptr_type(inkwell::AddressSpace::default()),
                "resume_cast",
            )
            .map_err(|e| format!("Failed to cast resume fn: {}", e))?;

        // Call runtime_spawn_global(runtime, resume_fn, state)
        self.builder
            .build_call(
                spawn_fn,
                &[
                    runtime_handle.into(),
                    resume_void_ptr.into(),
                    state_alloc_ptr.into(),
                ],
                "spawn",
            )
            .map_err(|e| format!("Failed to call runtime_spawn_global: {}", e))?;

        // Return appropriate value based on function return type
        // Async functions should return Future<T> (opaque pointer to runtime future handle)
        // For now, return null pointer as placeholder - runtime will manage the actual future
        if let Some(_ret_type) = &func.return_type {
            // TODO: Allocate and return actual Future<T> handle from runtime
            // For now, return null pointer (void*)
            let null_ptr = self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .const_null();
            self.builder
                .build_return(Some(&null_ptr))
                .map_err(|e| format!("Failed to build wrapper return: {}", e))?;
        } else {
            self.builder
                .build_return(None)
                .map_err(|e| format!("Failed to build wrapper return: {}", e))?;
        }

        Ok(())
    }

    pub(crate) fn get_or_declare_malloc(&mut self) -> FunctionValue<'ctx> {
        if let Some(malloc) = self.module.get_function("malloc") {
            return malloc;
        }

        let i64_type = self.context.i64_type();
        let i8_ptr = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let malloc_type = i8_ptr.fn_type(&[i64_type.into()], false);
        self.module.add_function("malloc", malloc_type, None)
    }

    pub(crate) fn get_or_declare_worker_await(&mut self) -> FunctionValue<'ctx> {
        if let Some(worker_await) = self.module.get_function("worker_await_after") {
            return worker_await;
        }

        // void worker_await_after(WorkerContext* context, uint64_t millis);
        let worker_ctx_ptr = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type();

        let worker_await_type = void_type.fn_type(&[worker_ctx_ptr.into(), i64_type.into()], false);
        self.module
            .add_function("worker_await_after", worker_await_type, None)
    }

    pub(crate) fn get_or_declare_runtime_spawn(&mut self) -> FunctionValue<'ctx> {
        if let Some(spawn) = self.module.get_function("runtime_spawn_global") {
            return spawn;
        }

        // void runtime_spawn_global(Runtime* runtime, coro_resume_func resume_fn, void* coro_data);
        let runtime_ptr = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let fn_ptr = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let void_ptr = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let void_type = self.context.void_type();

        let spawn_type =
            void_type.fn_type(&[runtime_ptr.into(), fn_ptr.into(), void_ptr.into()], false);
        self.module
            .add_function("runtime_spawn_global", spawn_type, None)
    }

    pub(crate) fn get_or_declare_runtime_create(&mut self) -> FunctionValue<'ctx> {
        if let Some(create) = self.module.get_function("runtime_create") {
            return create;
        }

        // Runtime* runtime_create(int num_workers);
        let runtime_ptr = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let i32_type = self.context.i32_type();

        let create_type = runtime_ptr.fn_type(&[i32_type.into()], false);
        self.module
            .add_function("runtime_create", create_type, None)
    }

    pub(crate) fn get_or_declare_runtime_run(&mut self) -> FunctionValue<'ctx> {
        if let Some(run) = self.module.get_function("runtime_run") {
            return run;
        }

        // void runtime_run(Runtime* runtime);
        let runtime_ptr = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let void_type = self.context.void_type();

        let run_type = void_type.fn_type(&[runtime_ptr.into()], false);
        self.module.add_function("runtime_run", run_type, None)
    }

    pub(crate) fn get_or_declare_runtime_destroy(&mut self) -> FunctionValue<'ctx> {
        if let Some(destroy) = self.module.get_function("runtime_destroy") {
            return destroy;
        }

        // void runtime_destroy(Runtime* runtime);
        let runtime_ptr = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let void_type = self.context.void_type();

        let destroy_type = void_type.fn_type(&[runtime_ptr.into()], false);
        self.module
            .add_function("runtime_destroy", destroy_type, None)
    }
}
