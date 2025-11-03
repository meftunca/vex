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
            use inkwell::context::Context;
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

            // Create LLVM context and AST codegen
            let context = Context::create();
            let mut codegen = vex_compiler::ASTCodeGen::new(&context, filename);

            println!("📦 Compiling {}...", input.display());

            // Parse the .vx file
            let source = std::fs::read_to_string(&input)?;
            let mut parser = vex_parser::Parser::new(&source)
                .map_err(|e| anyhow::anyhow!("⚠️  Lexer error: {}", e))?;

            let mut ast = parser
                .parse_file()
                .map_err(|e| anyhow::anyhow!("⚠️  Parse error: {}", e))?;

            println!("   ✅ Parsed {} successfully", filename);

            // Run borrow checker (Phase 1: Immutability)
            println!("   🔍 Running borrow checker...");
            let mut borrow_checker = vex_compiler::BorrowChecker::new();
            if let Err(e) = borrow_checker.check_program(&ast) {
                anyhow::bail!("⚠️  Borrow checker error: {}", e);
            }
            println!("   ✅ Borrow check passed");

            // Resolve imports if any
            if !ast.imports.is_empty() {
                println!("   📦 Resolving {} import(s)...", ast.imports.len());

                let std_lib_path = PathBuf::from("vex-libs/std");
                let mut resolver = vex_compiler::ModuleResolver::new(std_lib_path);

                for import in &ast.imports {
                    let module_path = &import.module;
                    println!(
                        "      • Loading module: {} ({:?})",
                        module_path, import.kind
                    );

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
                            println!("      ✓ Loaded {}", module_path);
                        }
                        Err(e) => {
                            anyhow::bail!("⚠️  Import error for '{}': {}", module_path, e);
                        }
                    }
                }

                println!("   ✅ All imports resolved");
            }

            // Compile AST to LLVM IR
            codegen
                .compile_program(&ast)
                .map_err(|e| anyhow::anyhow!(e))?;

            // Verify
            codegen.verify_and_print().map_err(|e| anyhow::anyhow!(e))?;

            if emit_llvm {
                // Save LLVM IR
                println!("\n🔍 Generated LLVM IR:");
                println!("{}", codegen.module.print_to_string());

                let ir_path = format!("{}.ll", filename);
                std::fs::write(&ir_path, codegen.module.print_to_string().to_string())?;
                println!("✓ LLVM IR saved to: {}", ir_path);
            }

            // Compile to object file
            let obj_path = format!("{}.o", filename);
            codegen
                .compile_to_object(std::path::Path::new(&obj_path))
                .map_err(|e| anyhow::anyhow!(e))?;
            println!("✓ Object file generated: {}", obj_path);

            // Link with clang
            println!("🔗 Linking...");
            let link_result = Command::new("clang")
                .arg(&obj_path)
                .arg("-o")
                .arg(&output_path)
                .output()?;

            if !link_result.status.success() {
                anyhow::bail!(
                    "Linking failed: {}",
                    String::from_utf8_lossy(&link_result.stderr)
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

            // Link
            let link_result = Command::new("clang")
                .arg(&obj_path)
                .arg("-o")
                .arg(&temp_output)
                .output()?;

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
