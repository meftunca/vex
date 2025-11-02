//! LLVM Code Generator for Vex
//! Compiles hardcoded programs to native machine code

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType};
use inkwell::values::{BasicValue, FunctionValue, IntValue};
use inkwell::IntPredicate;
use inkwell::OptimizationLevel;
use std::path::Path;

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        Self {
            context,
            module,
            builder,
        }
    }

    /// Compile hello world program
    pub fn compile_hello_world(&self) -> Result<(), String> {
        // Declare printf from C stdlib
        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
        let i32_type = self.context.i32_type();
        
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        let printf = self.module.add_function("printf", printf_type, None);

        // Create main function
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);
        let basic_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(basic_block);

        // Create string constant "Hello from Vex!\n"
        let hello_str = self.builder.build_global_string_ptr(
            "Hello from Vex!\n",
            "hello_str",
        ).map_err(|e| format!("Failed to create string: {:?}", e))?;

        // Call printf
        self.builder.build_call(
            printf,
            &[hello_str.as_pointer_value().into()],
            "printf_call",
        ).map_err(|e| format!("Failed to build call: {:?}", e))?;

        // Return 0
        let zero = i32_type.const_int(0, false);
        self.builder.build_return(Some(&zero))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        Ok(())
    }

    /// Compile fibonacci function
    pub fn compile_fibonacci(&self) -> Result<(), String> {
        let i32_type = self.context.i32_type();
        let i64_type = self.context.i64_type();
        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());

        // Declare printf
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        let printf = self.module.add_function("printf", printf_type, None);

        // Create fibonacci function: fn fib(n: i64) -> i64
        let fib_type = i64_type.fn_type(&[i64_type.into()], false);
        let fib_fn = self.module.add_function("fib", fib_type, None);
        
        let entry = self.context.append_basic_block(fib_fn, "entry");
        let then_block = self.context.append_basic_block(fib_fn, "then");
        let else_block = self.context.append_basic_block(fib_fn, "else");

        self.builder.position_at_end(entry);
        
        let n = fib_fn.get_nth_param(0).unwrap().into_int_value();
        
        // if n <= 1
        let cond = self.builder.build_int_compare(
            IntPredicate::SLE,
            n,
            i64_type.const_int(1, false),
            "cmp",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        
        self.builder.build_conditional_branch(cond, then_block, else_block)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // then: return n
        self.builder.position_at_end(then_block);
        self.builder.build_return(Some(&n))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        // else: return fib(n-1) + fib(n-2)
        self.builder.position_at_end(else_block);
        
        let n_minus_1 = self.builder.build_int_sub(
            n,
            i64_type.const_int(1, false),
            "n_minus_1",
        ).map_err(|e| format!("Failed to build sub: {:?}", e))?;
        
        let n_minus_2 = self.builder.build_int_sub(
            n,
            i64_type.const_int(2, false),
            "n_minus_2",
        ).map_err(|e| format!("Failed to build sub: {:?}", e))?;

        let call1 = self.builder.build_call(fib_fn, &[n_minus_1.into()], "fib_call1")
            .map_err(|e| format!("Failed to build call: {:?}", e))?;
        let result1 = call1.try_as_basic_value().left().unwrap().into_int_value();

        let call2 = self.builder.build_call(fib_fn, &[n_minus_2.into()], "fib_call2")
            .map_err(|e| format!("Failed to build call: {:?}", e))?;
        let result2 = call2.try_as_basic_value().left().unwrap().into_int_value();

        let sum = self.builder.build_int_add(result1, result2, "sum")
            .map_err(|e| format!("Failed to build add: {:?}", e))?;
        
        self.builder.build_return(Some(&sum))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        // Create main function
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);
        let main_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(main_block);

        // Call fib(10)
        let ten = i64_type.const_int(10, false);
        let fib_call = self.builder.build_call(fib_fn, &[ten.into()], "fib_result")
            .map_err(|e| format!("Failed to build call: {:?}", e))?;
        let fib_result = fib_call.try_as_basic_value().left().unwrap().into_int_value();

        // Print result: printf("fibonacci(10) = %lld\n", result)
        let format_str = self.builder.build_global_string_ptr(
            "fibonacci(10) = %lld\n",
            "format_str",
        ).map_err(|e| format!("Failed to create string: {:?}", e))?;

        self.builder.build_call(
            printf,
            &[format_str.as_pointer_value().into(), fib_result.into()],
            "printf_call",
        ).map_err(|e| format!("Failed to build call: {:?}", e))?;

        // Return 0
        let zero = i32_type.const_int(0, false);
        self.builder.build_return(Some(&zero))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        Ok(())
    }

    /// Compile GCD function (Euclidean algorithm with while loop)
    pub fn compile_gcd(&self) -> Result<(), String> {
        let i32_type = self.context.i32_type();
        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());

        // Declare printf
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        let printf = self.module.add_function("printf", printf_type, None);

        // Create gcd function: fn gcd(a: i32, b: i32) -> i32
        let gcd_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
        let gcd_fn = self.module.add_function("gcd", gcd_type, None);
        
        let entry = self.context.append_basic_block(gcd_fn, "entry");
        let loop_header = self.context.append_basic_block(gcd_fn, "loop_header");
        let loop_body = self.context.append_basic_block(gcd_fn, "loop_body");
        let loop_exit = self.context.append_basic_block(gcd_fn, "loop_exit");

        // Entry: allocate stack variables for a and b
        self.builder.position_at_end(entry);
        let a_ptr = self.builder.build_alloca(i32_type, "a_ptr")
            .map_err(|e| format!("Failed to build alloca: {:?}", e))?;
        let b_ptr = self.builder.build_alloca(i32_type, "b_ptr")
            .map_err(|e| format!("Failed to build alloca: {:?}", e))?;
        
        let a_param = gcd_fn.get_nth_param(0).unwrap().into_int_value();
        let b_param = gcd_fn.get_nth_param(1).unwrap().into_int_value();
        
        self.builder.build_store(a_ptr, a_param)
            .map_err(|e| format!("Failed to build store: {:?}", e))?;
        self.builder.build_store(b_ptr, b_param)
            .map_err(|e| format!("Failed to build store: {:?}", e))?;
        
        self.builder.build_unconditional_branch(loop_header)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // Loop header: check b != 0
        self.builder.position_at_end(loop_header);
        let b_val = self.builder.build_load(i32_type, b_ptr, "b_val")
            .map_err(|e| format!("Failed to build load: {:?}", e))?
            .into_int_value();
        
        let cond = self.builder.build_int_compare(
            IntPredicate::NE,
            b_val,
            i32_type.const_int(0, false),
            "cmp",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        
        self.builder.build_conditional_branch(cond, loop_body, loop_exit)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // Loop body: temp = b; b = a % b; a = temp
        self.builder.position_at_end(loop_body);
        let a_val = self.builder.build_load(i32_type, a_ptr, "a_val")
            .map_err(|e| format!("Failed to build load: {:?}", e))?
            .into_int_value();
        let b_val = self.builder.build_load(i32_type, b_ptr, "b_val")
            .map_err(|e| format!("Failed to build load: {:?}", e))?
            .into_int_value();
        
        let temp = b_val;
        let remainder = self.builder.build_int_signed_rem(a_val, b_val, "remainder")
            .map_err(|e| format!("Failed to build rem: {:?}", e))?;
        
        self.builder.build_store(b_ptr, remainder)
            .map_err(|e| format!("Failed to build store: {:?}", e))?;
        self.builder.build_store(a_ptr, temp)
            .map_err(|e| format!("Failed to build store: {:?}", e))?;
        
        self.builder.build_unconditional_branch(loop_header)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // Loop exit: return a
        self.builder.position_at_end(loop_exit);
        let final_a = self.builder.build_load(i32_type, a_ptr, "final_a")
            .map_err(|e| format!("Failed to build load: {:?}", e))?
            .into_int_value();
        
        self.builder.build_return(Some(&final_a))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        // Create main function
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);
        let main_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(main_block);

        // Call gcd(48, 18)
        let a = i32_type.const_int(48, false);
        let b = i32_type.const_int(18, false);
        let gcd_call = self.builder.build_call(gcd_fn, &[a.into(), b.into()], "gcd_result")
            .map_err(|e| format!("Failed to build call: {:?}", e))?;
        let gcd_result = gcd_call.try_as_basic_value().left().unwrap().into_int_value();

        // Print result
        let format_str = self.builder.build_global_string_ptr(
            "gcd(48, 18) = %d\n",
            "format_str",
        ).map_err(|e| format!("Failed to create string: {:?}", e))?;

        self.builder.build_call(
            printf,
            &[format_str.as_pointer_value().into(), gcd_result.into()],
            "printf_call",
        ).map_err(|e| format!("Failed to build call: {:?}", e))?;

        // Return 0
        let zero = i32_type.const_int(0, false);
        self.builder.build_return(Some(&zero))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        Ok(())
    }

    /// Compile factorial function
    pub fn compile_factorial(&self) -> Result<(), String> {
        let i32_type = self.context.i32_type();
        let i64_type = self.context.i64_type();
        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());

        // Declare printf
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        let printf = self.module.add_function("printf", printf_type, None);

        // Create factorial function: fn factorial(n: i64) -> i64
        let fact_type = i64_type.fn_type(&[i64_type.into()], false);
        let fact_fn = self.module.add_function("factorial", fact_type, None);
        
        let entry = self.context.append_basic_block(fact_fn, "entry");
        let then_block = self.context.append_basic_block(fact_fn, "then");
        let else_block = self.context.append_basic_block(fact_fn, "else");

        self.builder.position_at_end(entry);
        
        let n = fact_fn.get_nth_param(0).unwrap().into_int_value();
        
        // if n <= 1
        let cond = self.builder.build_int_compare(
            IntPredicate::SLE,
            n,
            i64_type.const_int(1, false),
            "cmp",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        
        self.builder.build_conditional_branch(cond, then_block, else_block)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // then: return 1
        self.builder.position_at_end(then_block);
        let one = i64_type.const_int(1, false);
        self.builder.build_return(Some(&one))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        // else: return n * factorial(n-1)
        self.builder.position_at_end(else_block);
        
        let n_minus_1 = self.builder.build_int_sub(
            n,
            i64_type.const_int(1, false),
            "n_minus_1",
        ).map_err(|e| format!("Failed to build sub: {:?}", e))?;

        let call = self.builder.build_call(fact_fn, &[n_minus_1.into()], "fact_call")
            .map_err(|e| format!("Failed to build call: {:?}", e))?;
        let result = call.try_as_basic_value().left().unwrap().into_int_value();

        let product = self.builder.build_int_mul(n, result, "product")
            .map_err(|e| format!("Failed to build mul: {:?}", e))?;
        
        self.builder.build_return(Some(&product))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        // Create main function
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);
        let main_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(main_block);

        // Call factorial(5)
        let five = i64_type.const_int(5, false);
        let fact_call = self.builder.build_call(fact_fn, &[five.into()], "fact_result")
            .map_err(|e| format!("Failed to build call: {:?}", e))?;
        let fact_result = fact_call.try_as_basic_value().left().unwrap().into_int_value();

        // Print result
        let format_str = self.builder.build_global_string_ptr(
            "factorial(5) = %lld\n",
            "format_str",
        ).map_err(|e| format!("Failed to create string: {:?}", e))?;

        self.builder.build_call(
            printf,
            &[format_str.as_pointer_value().into(), fact_result.into()],
            "printf_call",
        ).map_err(|e| format!("Failed to build call: {:?}", e))?;

        // Return 0
        let zero = i32_type.const_int(0, false);
        self.builder.build_return(Some(&zero))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        Ok(())
    }

    /// Compile prime number checker (with boolean return and complex while loop)
    pub fn compile_prime(&self) -> Result<(), String> {
        let i1_type = self.context.bool_type();
        let i32_type = self.context.i32_type();
        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());

        // Declare printf
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        let printf = self.module.add_function("printf", printf_type, None);

        // Create is_prime function: fn is_prime(n: i32) -> bool
        let prime_type = i1_type.fn_type(&[i32_type.into()], false);
        let prime_fn = self.module.add_function("is_prime", prime_type, None);
        
        let entry = self.context.append_basic_block(prime_fn, "entry");
        let check_le_1 = self.context.append_basic_block(prime_fn, "check_le_1");
        let check_le_3 = self.context.append_basic_block(prime_fn, "check_le_3");
        let check_even = self.context.append_basic_block(prime_fn, "check_even");
        let loop_init = self.context.append_basic_block(prime_fn, "loop_init");
        let loop_header = self.context.append_basic_block(prime_fn, "loop_header");
        let loop_body = self.context.append_basic_block(prime_fn, "loop_body");
        let loop_continue = self.context.append_basic_block(prime_fn, "loop_continue");
        let return_true = self.context.append_basic_block(prime_fn, "return_true");
        let return_false = self.context.append_basic_block(prime_fn, "return_false");

        self.builder.position_at_end(entry);
        let n = prime_fn.get_nth_param(0).unwrap().into_int_value();
        self.builder.build_unconditional_branch(check_le_1)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // if n <= 1: return false
        self.builder.position_at_end(check_le_1);
        let cond1 = self.builder.build_int_compare(
            IntPredicate::SLE,
            n,
            i32_type.const_int(1, false),
            "le_1",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        self.builder.build_conditional_branch(cond1, return_false, check_le_3)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // if n <= 3: return true
        self.builder.position_at_end(check_le_3);
        let cond2 = self.builder.build_int_compare(
            IntPredicate::SLE,
            n,
            i32_type.const_int(3, false),
            "le_3",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        self.builder.build_conditional_branch(cond2, return_true, check_even)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // if n % 2 == 0: return false
        self.builder.position_at_end(check_even);
        let mod2 = self.builder.build_int_signed_rem(n, i32_type.const_int(2, false), "mod2")
            .map_err(|e| format!("Failed to build rem: {:?}", e))?;
        let is_even = self.builder.build_int_compare(
            IntPredicate::EQ,
            mod2,
            i32_type.const_int(0, false),
            "is_even",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        self.builder.build_conditional_branch(is_even, return_false, loop_init)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // Loop initialization: i = 3
        self.builder.position_at_end(loop_init);
        let i_ptr = self.builder.build_alloca(i32_type, "i_ptr")
            .map_err(|e| format!("Failed to build alloca: {:?}", e))?;
        self.builder.build_store(i_ptr, i32_type.const_int(3, false))
            .map_err(|e| format!("Failed to build store: {:?}", e))?;
        self.builder.build_unconditional_branch(loop_header)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // Loop header: while i * i <= n
        self.builder.position_at_end(loop_header);
        let i_val = self.builder.build_load(i32_type, i_ptr, "i_val")
            .map_err(|e| format!("Failed to build load: {:?}", e))?
            .into_int_value();
        let i_squared = self.builder.build_int_mul(i_val, i_val, "i_squared")
            .map_err(|e| format!("Failed to build mul: {:?}", e))?;
        let loop_cond = self.builder.build_int_compare(
            IntPredicate::SLE,
            i_squared,
            n,
            "loop_cond",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        self.builder.build_conditional_branch(loop_cond, loop_body, return_true)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // Loop body: if n % i == 0: return false
        self.builder.position_at_end(loop_body);
        let i_val2 = self.builder.build_load(i32_type, i_ptr, "i_val2")
            .map_err(|e| format!("Failed to build load: {:?}", e))?
            .into_int_value();
        let remainder = self.builder.build_int_signed_rem(n, i_val2, "remainder")
            .map_err(|e| format!("Failed to build rem: {:?}", e))?;
        let is_divisible = self.builder.build_int_compare(
            IntPredicate::EQ,
            remainder,
            i32_type.const_int(0, false),
            "is_divisible",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        self.builder.build_conditional_branch(is_divisible, return_false, loop_continue)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // Loop continue: i = i + 2
        self.builder.position_at_end(loop_continue);
        let i_val3 = self.builder.build_load(i32_type, i_ptr, "i_val3")
            .map_err(|e| format!("Failed to build load: {:?}", e))?
            .into_int_value();
        let i_next = self.builder.build_int_add(i_val3, i32_type.const_int(2, false), "i_next")
            .map_err(|e| format!("Failed to build add: {:?}", e))?;
        self.builder.build_store(i_ptr, i_next)
            .map_err(|e| format!("Failed to build store: {:?}", e))?;
        self.builder.build_unconditional_branch(loop_header)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // Return blocks
        self.builder.position_at_end(return_true);
        self.builder.build_return(Some(&i1_type.const_int(1, false)))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        self.builder.position_at_end(return_false);
        self.builder.build_return(Some(&i1_type.const_int(0, false)))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        // Create main function
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);
        let main_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(main_block);

        // Test is_prime(17), is_prime(20), is_prime(97)
        let tests = vec![
            ("is_prime(17)", 17),
            ("is_prime(20)", 20),
            ("is_prime(97)", 97),
        ];

        for (name, val) in tests {
            let test_val = i32_type.const_int(val, false);
            let call = self.builder.build_call(prime_fn, &[test_val.into()], "prime_result")
                .map_err(|e| format!("Failed to build call: {:?}", e))?;
            let result = call.try_as_basic_value().left().unwrap().into_int_value();
            
            // Convert bool to i32 for printing
            let result_i32 = self.builder.build_int_z_extend(result, i32_type, "result_i32")
                .map_err(|e| format!("Failed to build zext: {:?}", e))?;

            let format_str = self.builder.build_global_string_ptr(
                &format!("{} = %d\n", name),
                "format_str",
            ).map_err(|e| format!("Failed to create string: {:?}", e))?;

            self.builder.build_call(
                printf,
                &[format_str.as_pointer_value().into(), result_i32.into()],
                "printf_call",
            ).map_err(|e| format!("Failed to build call: {:?}", e))?;
        }

        // Return 0
        let zero = i32_type.const_int(0, false);
        self.builder.build_return(Some(&zero))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        Ok(())
    }

    /// Compile power function (exponentiation with optimization)
    pub fn compile_power(&self) -> Result<(), String> {
        let i32_type = self.context.i32_type();
        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());

        // Declare printf
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        let printf = self.module.add_function("printf", printf_type, None);

        // Create power function: fn power(base: i32, exp: i32) -> i32
        let power_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
        let power_fn = self.module.add_function("power", power_type, None);
        
        let entry = self.context.append_basic_block(power_fn, "entry");
        let exp_zero = self.context.append_basic_block(power_fn, "exp_zero");
        let exp_one = self.context.append_basic_block(power_fn, "exp_one");
        let recurse = self.context.append_basic_block(power_fn, "recurse");
        let check_even = self.context.append_basic_block(power_fn, "check_even");
        let even_case = self.context.append_basic_block(power_fn, "even_case");
        let odd_case = self.context.append_basic_block(power_fn, "odd_case");

        self.builder.position_at_end(entry);
        
        let base = power_fn.get_nth_param(0).unwrap().into_int_value();
        let exp = power_fn.get_nth_param(1).unwrap().into_int_value();
        
        // if exp == 0
        let is_zero = self.builder.build_int_compare(
            IntPredicate::EQ,
            exp,
            i32_type.const_int(0, false),
            "is_zero",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        
        self.builder.build_conditional_branch(is_zero, exp_zero, exp_one)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // exp == 0: return 1
        self.builder.position_at_end(exp_zero);
        let one = i32_type.const_int(1, false);
        self.builder.build_return(Some(&one))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        // Check if exp == 1
        self.builder.position_at_end(exp_one);
        let is_one = self.builder.build_int_compare(
            IntPredicate::EQ,
            exp,
            i32_type.const_int(1, false),
            "is_one",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        
        self.builder.build_conditional_branch(is_one, recurse, recurse)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // Recursive case
        self.builder.position_at_end(recurse);
        
        // Check exp == 1 again for actual return
        let is_one_check = self.builder.build_int_compare(
            IntPredicate::EQ,
            exp,
            i32_type.const_int(1, false),
            "is_one_check",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        
        let return_base_block = self.context.append_basic_block(power_fn, "return_base");
        self.builder.build_conditional_branch(is_one_check, return_base_block, check_even)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;
        
        self.builder.position_at_end(return_base_block);
        self.builder.build_return(Some(&base))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        // Compute half = power(base, exp / 2)
        self.builder.position_at_end(check_even);
        let exp_div_2 = self.builder.build_int_signed_div(exp, i32_type.const_int(2, false), "exp_div_2")
            .map_err(|e| format!("Failed to build div: {:?}", e))?;
        
        let half_call = self.builder.build_call(power_fn, &[base.into(), exp_div_2.into()], "half_call")
            .map_err(|e| format!("Failed to build call: {:?}", e))?;
        let half = half_call.try_as_basic_value().left().unwrap().into_int_value();

        // Check if exp is even
        let exp_mod_2 = self.builder.build_int_signed_rem(exp, i32_type.const_int(2, false), "exp_mod_2")
            .map_err(|e| format!("Failed to build rem: {:?}", e))?;
        
        let is_even = self.builder.build_int_compare(
            IntPredicate::EQ,
            exp_mod_2,
            i32_type.const_int(0, false),
            "is_even",
        ).map_err(|e| format!("Failed to build compare: {:?}", e))?;
        
        self.builder.build_conditional_branch(is_even, even_case, odd_case)
            .map_err(|e| format!("Failed to build branch: {:?}", e))?;

        // Even case: return half * half
        self.builder.position_at_end(even_case);
        let result_even = self.builder.build_int_mul(half, half, "result_even")
            .map_err(|e| format!("Failed to build mul: {:?}", e))?;
        self.builder.build_return(Some(&result_even))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        // Odd case: return base * half * half
        self.builder.position_at_end(odd_case);
        let half_squared = self.builder.build_int_mul(half, half, "half_squared")
            .map_err(|e| format!("Failed to build mul: {:?}", e))?;
        let result_odd = self.builder.build_int_mul(base, half_squared, "result_odd")
            .map_err(|e| format!("Failed to build mul: {:?}", e))?;
        self.builder.build_return(Some(&result_odd))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        // Create main function
        let main_type = i32_type.fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_type, None);
        let main_block = self.context.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(main_block);

        // Call power(2, 10)
        let base_val = i32_type.const_int(2, false);
        let exp_val = i32_type.const_int(10, false);
        let power_call = self.builder.build_call(power_fn, &[base_val.into(), exp_val.into()], "power_result")
            .map_err(|e| format!("Failed to build call: {:?}", e))?;
        let power_result = power_call.try_as_basic_value().left().unwrap().into_int_value();

        // Print result
        let format_str = self.builder.build_global_string_ptr(
            "power(2, 10) = %d\n",
            "format_str",
        ).map_err(|e| format!("Failed to create string: {:?}", e))?;

        self.builder.build_call(
            printf,
            &[format_str.as_pointer_value().into(), power_result.into()],
            "printf_call",
        ).map_err(|e| format!("Failed to build call: {:?}", e))?;

        // Return 0
        let zero = i32_type.const_int(0, false);
        self.builder.build_return(Some(&zero))
            .map_err(|e| format!("Failed to build return: {:?}", e))?;

        Ok(())
    }

    /// Verify and print LLVM IR
    pub fn verify_and_print(&self) -> Result<(), String> {
        if self.module.verify().is_err() {
            return Err("Module verification failed".to_string());
        }
        
        println!("\n=== LLVM IR ===\n{}\n", self.module.print_to_string().to_string());
        Ok(())
    }

    /// Compile to object file
    pub fn compile_to_object(&self, output_path: &Path) -> Result<(), String> {
        use inkwell::targets::{
            CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
        };

        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| format!("Failed to initialize target: {}", e))?;

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| format!("Failed to get target: {}", e))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or("Failed to create target machine")?;

        target_machine
            .write_to_file(&self.module, FileType::Object, output_path)
            .map_err(|e| format!("Failed to write object file: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world_codegen() {
        let context = Context::create();
        let codegen = CodeGen::new(&context, "hello");
        assert!(codegen.compile_hello_world().is_ok());
        assert!(codegen.verify_and_print().is_ok());
    }

    #[test]
    fn test_fibonacci_codegen() {
        let context = Context::create();
        let codegen = CodeGen::new(&context, "fibonacci");
        assert!(codegen.compile_fibonacci().is_ok());
        assert!(codegen.verify_and_print().is_ok());
    }
}
