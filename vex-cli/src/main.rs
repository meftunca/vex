use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "vex")]
#[command(version = "0.2.0")]
#[command(about = "Vex Programming Language Compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Vex source file
    Compile {
        /// Input .vx file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Output file
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,

        /// Enable SIMD optimizations
        #[arg(long)]
        simd: bool,

        /// Enable GPU support
        #[arg(long)]
        gpu: bool,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "2")]
        opt_level: u8,

        /// Emit LLVM IR
        #[arg(long)]
        emit_llvm: bool,

        /// Emit SPIR-V (for GPU functions)
        #[arg(long)]
        emit_spirv: bool,
    },

    /// Run a Vex source file (compile and execute)
    Run {
        /// Input .vx file or code string with -c
        #[arg(value_name = "INPUT")]
        input: Option<PathBuf>,

        /// Execute code from string (like node -c)
        #[arg(short, long, value_name = "CODE")]
        code: Option<String>,

        /// Arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Check syntax without compiling
    Check {
        /// Input .vx file
        #[arg(value_name = "INPUT")]
        input: PathBuf,
    },

    /// Format Vex source code
    Format {
        /// Input .vx file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Format in place
        #[arg(short, long)]
        in_place: bool,
    },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            simd,
            gpu,
            opt_level,
            emit_llvm,
            emit_spirv: _,
        } => {
            use inkwell::targets::{FileType, Target};
            use std::process::Command;

            log::info!("Compiling: {:?}", input);
            log::info!("SIMD: {}, GPU: {}, Opt Level: {}", simd, gpu, opt_level);

            let filename = input
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("program");

            // Create vex-builds directory if it doesn't exist
            std::fs::create_dir_all("vex-builds")?;

            let output_path =
                output.unwrap_or_else(|| PathBuf::from(format!("vex-builds/{}", filename)));

            // --- Full Compilation Pipeline ---
            let source = std::fs::read_to_string(&input)?;
            let mut parser = vex_parser::Parser::new(&source)?;
            let mut ast = parser.parse_file()?;
            println!("   ✅ Parsed {} successfully", filename);

            let mut borrow_checker = vex_compiler::BorrowChecker::new();
            borrow_checker.check_program(&mut ast)?;
            println!("   ✅ Borrow check passed");

            let context = inkwell::context::Context::create();
            let mut codegen = vex_compiler::ASTCodeGen::new(&context, filename);
            codegen
                .compile_program(&ast)
                .map_err(|e| anyhow::anyhow!(e))?;

            // Link the final executable
            println!("   🔗 Linking executable...");
            let obj_path = PathBuf::from(format!("vex-builds/{}.o", filename));

            Target::initialize_native(&inkwell::targets::InitializationConfig::default())
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            let target_triple = inkwell::targets::TargetMachine::get_default_triple();
            let target =
                Target::from_triple(&target_triple).map_err(|e| anyhow::anyhow!(e.to_string()))?;
            let target_machine = target
                .create_target_machine(
                    &target_triple,
                    "generic",
                    "",
                    inkwell::OptimizationLevel::Default,
                    inkwell::targets::RelocMode::Default,
                    inkwell::targets::CodeModel::Default,
                )
                .ok_or_else(|| anyhow::anyhow!("Unable to create target machine"))?;

            target_machine
                .write_to_file(&codegen.module, FileType::Object, &obj_path)
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

            // Link the object file
            let mut command = Command::new("clang");
            command.arg(&obj_path).arg("-o").arg(&output_path);

            // Add linker arguments from vex-runtime
            let linker_args = vex_runtime::get_linker_args();
            println!("cargo:warning=CLI received linker args: '{}'", linker_args);
            for arg in linker_args.split_whitespace() {
                println!("cargo:warning=CLI adding linker arg: '{}'", arg);
                command.arg(arg);
            }

            let output = command.output().map_err(|e| anyhow::anyhow!(e))?;

            if !output.status.success() {
                anyhow::bail!(
                    "Error: Linking failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            println!("✓ Compilation successful!");
            println!("  Output: {}", output_path.display());
            println!("\n▶️  Run with: ./{}", output_path.display());

            // Clean up object file
            std::fs::remove_file(&obj_path).ok();

            Ok(())
        }
        Commands::Run { input, code, args } => {
            use inkwell::context::Context;
            use std::process::Command;

            // Handle both file input and direct code execution
            let (source, filename): (String, String) = if let Some(code_str) = code {
                // Direct code execution: vex run -c "print(42);"
                println!("🚀 Executing code snippet");
                (code_str, "inline_code".to_string())
            } else if let Some(input_path) = input {
                // File execution: vex run file.vx
                println!("🚀 Running: {:?}", input_path);
                let fname = input_path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow::anyhow!("Invalid input filename"))?;
                let src = std::fs::read_to_string(&input_path)?;
                (src, fname)
            } else {
                anyhow::bail!("Either INPUT file or -c CODE must be provided");
            };

            // Create a temporary output path
            let temp_output = std::env::temp_dir().join(format!("vex_run_{}", filename));

            log::info!("Compiling to temporary: {:?}", temp_output);

            // Parse
            let mut parser = vex_parser::Parser::new(&source)?;
            let mut ast = parser.parse_file()?;

            println!("   ✅ Parsed {} successfully", filename);

            // Run borrow checker (Phase 1-5: Immutability, Moves, Borrows, Lifetimes, Closure Traits)
            println!("   🔍 Running borrow checker...");
            let mut borrow_checker = vex_compiler::BorrowChecker::new();
            if let Err(e) = borrow_checker.check_program(&mut ast) {
                anyhow::bail!("⚠️  Borrow checker error: {}", e);
            }
            println!("   ✅ Borrow check passed");

            // Codegen
            let context = Context::create();
            let mut codegen = vex_compiler::ASTCodeGen::new(&context, &filename);

            // Resolve imports if any
            if !ast.imports.is_empty() {
                let std_lib_path = PathBuf::from("vex-libs/std");
                let mut resolver = vex_compiler::ModuleResolver::new(std_lib_path);

                for import in &ast.imports {
                    let module_path = &import.module;

                    match resolver.load_module(module_path) {
                        Ok(module_ast) => {
                            match &import.kind {
                                vex_ast::ImportKind::Named => {
                                    // Named import: only import requested items
                                    if import.items.is_empty() {
                                        // If no specific items, import all
                                        for item in &module_ast.items {
                                            match item {
                                                vex_ast::Item::Function(func) => {
                                                    ast.items.push(vex_ast::Item::Function(
                                                        func.clone(),
                                                    ));
                                                }
                                                vex_ast::Item::Struct(struct_def) => {
                                                    ast.items.push(vex_ast::Item::Struct(
                                                        struct_def.clone(),
                                                    ));
                                                }
                                                _ => {}
                                            }
                                        }
                                    } else {
                                        // Import only specific items
                                        for requested in &import.items {
                                            for item in &module_ast.items {
                                                match item {
                                                    vex_ast::Item::Function(func)
                                                        if func.name == *requested =>
                                                    {
                                                        ast.items.push(vex_ast::Item::Function(
                                                            func.clone(),
                                                        ));
                                                    }
                                                    vex_ast::Item::Struct(s)
                                                        if s.name == *requested =>
                                                    {
                                                        ast.items
                                                            .push(vex_ast::Item::Struct(s.clone()));
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                vex_ast::ImportKind::Module => {
                                    // Module import: import all and track namespace
                                    let module_name = module_path
                                        .split(&['/', ':'][..])
                                        .last()
                                        .unwrap_or(module_path);

                                    let mut imported_funcs = Vec::new();
                                    for item in &module_ast.items {
                                        match item {
                                            vex_ast::Item::Function(func) => {
                                                ast.items
                                                    .push(vex_ast::Item::Function(func.clone()));
                                                imported_funcs.push(func.name.clone());
                                            }
                                            vex_ast::Item::Struct(struct_def) => {
                                                ast.items.push(vex_ast::Item::Struct(
                                                    struct_def.clone(),
                                                ));
                                            }
                                            _ => {}
                                        }
                                    }
                                    // Track module namespace for codegen
                                    codegen.register_module_namespace(
                                        module_name.to_string(),
                                        imported_funcs,
                                    );
                                }
                                vex_ast::ImportKind::Namespace(alias) => {
                                    // Namespace import: import all with alias
                                    let mut imported_funcs = Vec::new();
                                    for item in &module_ast.items {
                                        match item {
                                            vex_ast::Item::Function(func) => {
                                                ast.items
                                                    .push(vex_ast::Item::Function(func.clone()));
                                                imported_funcs.push(func.name.clone());
                                            }
                                            vex_ast::Item::Struct(struct_def) => {
                                                ast.items.push(vex_ast::Item::Struct(
                                                    struct_def.clone(),
                                                ));
                                            }
                                            _ => {}
                                        }
                                    }
                                    // Track with alias
                                    codegen
                                        .register_module_namespace(alias.clone(), imported_funcs);
                                }
                            }
                        }
                        Err(e) => {
                            anyhow::bail!("⚠️  Import error for '{}': {}", module_path, e);
                        }
                    }
                }
            }

            codegen
                .compile_program(&ast)
                .map_err(|e| anyhow::anyhow!("Compilation error: {}", e))?;

            // Compile to object file
            let obj_path = temp_output.with_extension("o");
            codegen
                .compile_to_object(&obj_path)
                .map_err(|e| anyhow::anyhow!("Object file generation error: {}", e))?;

            let mut command = Command::new("clang");
            command.arg(&obj_path).arg("-o").arg(&temp_output);

            // Add linker arguments from vex-runtime
            let linker_args = vex_runtime::get_linker_args();
            println!("  🔗 Linking with args: '{}'", linker_args);
            for arg in linker_args.split_whitespace() {
                command.arg(arg);
            }

            let link_result = command.output()?;

            if !link_result.status.success() {
                std::fs::remove_file(&obj_path).ok();
                anyhow::bail!(
                    "Linking failed: {}",
                    String::from_utf8_lossy(&link_result.stderr)
                );
            }

            // Execute the compiled program
            let mut child = Command::new(&temp_output).args(&args).spawn()?;

            let status = child.wait()?;

            // Cleanup
            std::fs::remove_file(&obj_path).ok();
            std::fs::remove_file(&temp_output).ok();

            if !status.success() {
                if let Some(code) = status.code() {
                    std::process::exit(code);
                } else {
                    anyhow::bail!("Program terminated by signal");
                }
            }

            Ok(())
        }
        Commands::Check { input } => {
            // TODO: Implement syntax checking
            println!("🔍 Checking: {:?}", input);
            let source = std::fs::read_to_string(&input)?;
            let mut parser = vex_parser::Parser::new(&source)?;

            match parser.parse_file() {
                Ok(_) => {
                    println!("✅ Syntax OK");
                    Ok(())
                }
                Err(e) => {
                    println!("❌ Parse error: {}", e);
                    Err(anyhow::anyhow!(e))
                }
            }
        }
        Commands::Format { input, in_place } => {
            // TODO: Implement formatter
            println!("✨ Formatting: {:?}, in_place: {}", input, in_place);
            anyhow::bail!("Format command not yet implemented");
        }
    }
}
