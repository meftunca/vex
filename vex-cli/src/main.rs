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
    /// Create a new Vex project
    New {
        /// Project name
        #[arg(value_name = "NAME")]
        name: String,

        /// Project path (default: ./<name>)
        #[arg(short, long, value_name = "PATH")]
        path: Option<PathBuf>,
    },

    /// Initialize vex.json in existing directory
    Init {
        /// Project path (default: current directory)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },

    /// Add a dependency
    Add {
        /// Package URL (e.g., github.com/user/repo@v1.0.0)
        #[arg(value_name = "PACKAGE")]
        package: String,

        /// Version (if not in package URL)
        #[arg(short, long, value_name = "VERSION")]
        version: Option<String>,
    },

    /// Remove a dependency
    Remove {
        /// Package name
        #[arg(value_name = "PACKAGE")]
        package: String,
    },

    /// List all dependencies
    List,

    /// Update all dependencies to latest versions
    Update,

    /// Clean cache and build artifacts
    Clean,

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

        /// Use lock file (CI mode - fails if lock file is invalid)
        #[arg(long)]
        locked: bool,

        /// Emit SPIR-V (for GPU functions)
        #[arg(long)]
        emit_spirv: bool,

        /// Output diagnostics as JSON (for IDE integration)
        #[arg(long)]
        json: bool,
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

        /// Output diagnostics as JSON (for IDE integration)
        #[arg(long)]
        json: bool,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "0")]
        opt_level: u8,
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

    /// Interactive REPL (Read-Eval-Print-Loop)
    Repl {
        /// Load file before starting REPL
        #[arg(short, long, value_name = "FILE")]
        load: Option<PathBuf>,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run tests
    Test {
        /// Specific test file or pattern (default: all tests)
        #[arg(value_name = "PATTERN")]
        pattern: Option<String>,

        /// Run tests verbosely
        #[arg(short, long)]
        verbose: bool,

        /// Disable parallel test execution
        #[arg(long)]
        no_parallel: bool,

        /// Custom timeout in seconds
        #[arg(long, value_name = "SECONDS")]
        timeout: Option<u64>,

        /// Run benchmarks instead of tests
        #[arg(long)]
        bench: bool,

        /// Benchmark execution time
        #[arg(long, value_name = "DURATION", default_value = "1s")]
        benchtime: String,

        /// Number of benchmark iterations
        #[arg(long, value_name = "N", default_value = "1")]
        count: u32,

        /// Show memory allocation statistics for benchmarks
        #[arg(long)]
        benchmem: bool,

        /// Generate coverage report
        #[arg(long)]
        coverage: bool,

        /// Coverage profile output file
        #[arg(long, value_name = "FILE")]
        coverprofile: Option<PathBuf>,

        /// Coverage mode: set, count, or atomic
        #[arg(long, value_name = "MODE", default_value = "set")]
        covermode: String,

        /// Run in short mode (skip slow tests)
        #[arg(long)]
        short: bool,

        /// Run fuzzing tests
        #[arg(long, value_name = "FUZZ_TARGET")]
        fuzz: Option<String>,

        /// Fuzzing execution time
        #[arg(long, value_name = "DURATION", default_value = "10s")]
        fuzztime: String,

        /// Filter tests by name (regex)
        #[arg(long, value_name = "REGEX")]
        run: Option<String>,
    },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, path } => {
            vex_pm::create_new_project(&name, path)?;
            Ok(())
        }

        Commands::Init { path } => {
            vex_pm::init_project(path)?;
            Ok(())
        }

        Commands::Add { package, version } => {
            vex_pm::add_dependency(&package, version.as_deref())?;
            Ok(())
        }

        Commands::Remove { package } => {
            vex_pm::remove_dependency(&package)?;
            Ok(())
        }

        Commands::List => {
            vex_pm::list_dependencies()?;
            Ok(())
        }

        Commands::Update => {
            vex_pm::update_dependencies()?;
            Ok(())
        }

        Commands::Clean => {
            vex_pm::clean_cache()?;
            Ok(())
        }

        Commands::Compile {
            input,
            output,
            simd,
            gpu,
            opt_level,
            emit_llvm,
            locked,
            emit_spirv,
            json,
        } => {
            // Resolve dependencies before compilation
            if let Err(e) = vex_pm::resolve_dependencies_for_build(locked) {
                if json {
                    println!("{{\"error\":\"Dependency resolution failed: {}\"}}}}", e);
                } else {
                    eprintln!("❌ Dependency resolution failed: {}", e);
                }
                return Err(anyhow::anyhow!("Dependency resolution failed: {}", e));
            }

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
            let input_str = input.to_str().unwrap_or("unknown.vx");
            let mut parser = vex_parser::Parser::new_with_file(input_str, &source)?;

            // Use error recovery to collect all parse errors
            let (ast_opt, parse_diagnostics) = parser.parse_with_recovery();
            
            // Display all parse diagnostics
            if !parse_diagnostics.is_empty() {
                if json {
                    // Output all diagnostics as JSON array
                    println!("{{\"diagnostics\":[");
                    for (i, diag) in parse_diagnostics.iter().enumerate() {
                        if i > 0 {
                            println!(",");
                        }
                        println!("  {{");
                        println!("    \"level\":\"{}\",", diag.level);
                        println!("    \"code\":\"{}\",", diag.code);
                        println!("    \"message\":\"{}\",", diag.message.replace('"', "\\\""));
                        println!("    \"file\":\"{}\",", diag.span.file);
                        println!("    \"line\":{},", diag.span.line);
                        println!("    \"column\":{}", diag.span.column);
                        println!("  }}");
                    }
                    println!("]}}");
                } else {
                    // Print all diagnostics with formatting
                    for diag in &parse_diagnostics {
                        eprintln!("{}", diag.format(&source));
                        eprintln!(); // Blank line between errors
                    }
                }
                
                // If parsing failed completely, abort
                if ast_opt.is_none() {
                    return Err(anyhow::anyhow!("Parse failed with {} error(s)", parse_diagnostics.len()));
                }
            }
            
            let mut ast = ast_opt.unwrap();

            // ⭐ INJECT EMBEDDED PRELUDE (Layer 1 - Self-hosted)
            ast = vex_compiler::inject_prelude_into_program(ast)
                .map_err(|e| anyhow::anyhow!("Failed to load prelude: {}", e))?;

            // Extract span map from parser
            let span_map = parser.take_span_map();

            println!("   ✅ Parsed {} successfully", filename);

            let mut borrow_checker = vex_compiler::BorrowChecker::new();
            if let Err(borrow_error) = borrow_checker.check_program(&mut ast) {
                // Convert borrow error to diagnostic
                let diagnostic = borrow_error.to_diagnostic();

                if json {
                    // Output as single diagnostic JSON
                    println!("{{\"diagnostics\":[{{");
                    println!("  \"level\":\"error\",");
                    println!("  \"code\":\"{}\",", diagnostic.code);
                    println!(
                        "  \"message\":\"{}\",",
                        diagnostic.message.replace('"', "\\\"")
                    );
                    println!("  \"file\":\"{}\",", diagnostic.span.file);
                    println!("  \"line\":{},", diagnostic.span.line);
                    println!("  \"column\":{}", diagnostic.span.column);
                    println!("}}]}}");
                } else {
                    eprintln!("{}", diagnostic.format(&source));
                }
                return Err(anyhow::anyhow!("Borrow check failed"));
            }
            println!("   ✅ Borrow check passed");

            // Run linter for warnings
            let mut linter = vex_compiler::Linter::new();
            let lint_warnings = linter.lint(&ast);
            
            if !lint_warnings.is_empty() {
                if json {
                    // Append warnings to JSON output
                    println!("{{\"warnings\":[");
                    for (i, warning) in lint_warnings.iter().enumerate() {
                        if i > 0 {
                            println!(",");
                        }
                        println!("  {{");
                        println!("    \"level\":\"warning\",");
                        println!("    \"code\":\"{}\",", warning.code);
                        println!("    \"message\":\"{}\",", warning.message.replace('"', "\\\""));
                        println!("    \"file\":\"{}\",", warning.span.file);
                        println!("    \"line\":{},", warning.span.line);
                        println!("    \"column\":{}", warning.span.column);
                        println!("  }}");
                    }
                    println!("]}}");
                } else {
                    // Print warnings with formatting
                    for warning in &lint_warnings {
                        eprintln!("{}", warning.format(&source));
                    }
                    eprintln!(); // Blank line after warnings
                }
            }

            let context = inkwell::context::Context::create();
            let mut codegen = vex_compiler::ASTCodeGen::new_with_source_file(
                &context, filename, span_map, input_str,
            );

            eprintln!("🔍 About to call compile_program");
            let compile_result = codegen.compile_program(&ast);
            eprintln!("🔍 compile_program returned: {:?}", compile_result.is_ok());

            // Print diagnostics based on output format
            if codegen.has_diagnostics() {
                if json {
                    println!("{}", codegen.diagnostics().to_json());
                } else {
                    codegen.diagnostics().print_all(&source);
                    codegen.diagnostics().print_summary();
                }
            }

            // Check compilation result and diagnostics
            if let Err(e) = compile_result {
                if codegen.has_errors() {
                    return Err(anyhow::anyhow!("Compilation failed with errors"));
                } else {
                    return Err(anyhow::anyhow!(e));
                }
            }

            if codegen.has_errors() {
                return Err(anyhow::anyhow!("Compilation failed with errors"));
            }

            // Print error/warning summary
            let error_count = parse_diagnostics.iter().filter(|d| d.level == vex_compiler::ErrorLevel::Error).count();
            let warning_count = lint_warnings.len();
            
            if !json && (error_count > 0 || warning_count > 0) {
                eprintln!();
                if error_count > 0 && warning_count > 0 {
                    eprintln!("error: aborting due to {} previous error(s); {} warning(s) emitted", error_count, warning_count);
                } else if error_count > 0 {
                    eprintln!("error: aborting due to {} previous error(s)", error_count);
                } else {
                    eprintln!("warning: {} warning(s) emitted", warning_count);
                }
            }

            eprintln!("🔍 About to initialize LLVM target");
            // Link the final executable
            println!("   🔗 Linking executable...");
            let obj_path = PathBuf::from(format!("vex-builds/{}.o", filename));

            eprintln!("🔍 Calling Target::initialize_native");
            Target::initialize_native(&inkwell::targets::InitializationConfig::default())
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            eprintln!("🔍 Target initialized");
            let target_triple = inkwell::targets::TargetMachine::get_default_triple();
            let target =
                Target::from_triple(&target_triple).map_err(|e| anyhow::anyhow!(e.to_string()))?;

            // Map CLI opt_level to LLVM OptimizationLevel
            let llvm_opt_level = match opt_level {
                0 => inkwell::OptimizationLevel::None,
                1 => inkwell::OptimizationLevel::Less,
                2 => inkwell::OptimizationLevel::Default,
                3 => inkwell::OptimizationLevel::Aggressive,
                _ => inkwell::OptimizationLevel::Default,
            };

            let target_machine = target
                .create_target_machine(
                    &target_triple,
                    "generic",
                    "",
                    llvm_opt_level,
                    inkwell::targets::RelocMode::Default,
                    inkwell::targets::CodeModel::Default,
                )
                .ok_or_else(|| anyhow::anyhow!("Unable to create target machine"))?;

            // Write LLVM IR, SPIR-V, or object file based on flags
            if emit_llvm {
                let ll_path = output_path.with_extension("ll");
                codegen
                    .module
                    .print_to_file(&ll_path)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                println!("✓ LLVM IR generated!");
                println!("  Output: {}", ll_path.display());
                println!("\n▶️  View with: cat {}", ll_path.display());
                return Ok(());
            }

            if emit_spirv {
                // SPIR-V emission for GPU code
                if !gpu {
                    if json {
                        println!("{{\"error\":\"SPIR-V emission requires --gpu flag\"}}");
                    } else {
                        eprintln!("⚠️  Warning: --emit-spirv requires --gpu flag");
                        eprintln!("   Use: vex compile --gpu --emit-spirv {}", input.display());
                    }
                    return Err(anyhow::anyhow!("SPIR-V emission requires GPU mode"));
                }

                let spirv_path = output_path.with_extension("spv");
                
                // TODO: Implement actual SPIR-V emission via LLVM SPIR-V backend
                // For now, emit LLVM IR with SPIR-V target triple
                let spirv_ll = output_path.with_extension("spirv.ll");
                codegen
                    .module
                    .print_to_file(&spirv_ll)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                
                if json {
                    println!("{{\"spirv_ir\":\"{}\"}}", spirv_ll.display());
                } else {
                    println!("✓ SPIR-V IR generated!");
                    println!("  Output: {}", spirv_ll.display());
                    println!("\n⚠️  Note: Full SPIR-V binary emission is not yet implemented");
                    println!("   Generated LLVM IR with GPU annotations instead");
                    println!("\n▶️  Convert with: llvm-spirv {} -o {}", spirv_ll.display(), spirv_path.display());
                }
                return Ok(());
            }

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

            // Add native library linker arguments from vex.json
            if let Ok(manifest_path) = std::env::current_dir().map(|d| d.join("vex.json")) {
                if manifest_path.exists() {
                    if let Ok(manifest) = vex_pm::Manifest::from_file(&manifest_path) {
                        if let Some(native_config) = manifest.get_native() {
                            let linker = vex_pm::NativeLinker::new(std::env::current_dir()?);
                            match linker.process(native_config) {
                                Ok(native_args) if !native_args.is_empty() => {
                                    println!("   🔗 Adding native libraries: {}", native_args);
                                    for arg in native_args.split_whitespace() {
                                        command.arg(arg);
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "⚠️  Warning: Failed to process native config: {}",
                                        e
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
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
        Commands::Run {
            input,
            code,
            args,
            json,
            opt_level,
        } => {
            use inkwell::context::Context;
            use std::process::Command;

            // Handle both file input and direct code execution
            let (source, filename, parser_file): (String, String, String) =
                if let Some(code_str) = code {
                    // Direct code execution: vex run -c "print(42);"
                    println!("🚀 Executing code snippet");
                    (
                        code_str,
                        "inline_code".to_string(),
                        "inline_code".to_string(),
                    )
                } else if let Some(ref input_path) = input {
                    // File execution: vex run file.vx
                    println!("🚀 Running: {:?}", input_path);
                    let fname = input_path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| anyhow::anyhow!("Invalid input filename"))?;
                    let src = std::fs::read_to_string(&input_path)?;
                    // Convert to absolute path for proper import resolution
                    let abs_path = std::fs::canonicalize(input_path)?;
                    let parser_f = abs_path.to_str().unwrap_or("unknown.vx").to_string();
                    (src, fname, parser_f)
                } else {
                    anyhow::bail!("Either INPUT file or -c CODE must be provided");
                };

            // Create a temporary output path
            let temp_output = std::env::temp_dir().join(format!("vex_run_{}", filename));

            log::info!("Compiling to temporary: {:?}", temp_output);

            // Parse
            let mut parser = vex_parser::Parser::new_with_file(&parser_file, &source)?;
            let mut ast = match parser.parse_file() {
                Ok(ast) => ast,
                Err(parse_error) => {
                    // Print parse error as formatted diagnostic
                    if let Some(diag) = parse_error.as_diagnostic() {
                        if json {
                            // Output single diagnostic as JSON
                            println!("{{\"diagnostics\":[{{");
                            println!("  \"level\":\"error\",");
                            println!("  \"code\":\"{}\",", diag.code);
                            println!("  \"message\":\"{}\",", diag.message.replace('"', "\\\""));
                            println!("  \"file\":\"{}\",", diag.span.file);
                            println!("  \"line\":{},", diag.span.line);
                            println!("  \"column\":{}", diag.span.column);
                            println!("}}]}}");
                        } else {
                            eprintln!("{}", diag.format(&source));
                        }
                    } else {
                        eprintln!("{}", parse_error);
                    }
                    return Err(anyhow::anyhow!("Parse failed"));
                }
            };

            // ⭐ INJECT EMBEDDED PRELUDE (Layer 1 - Self-hosted)
            ast = vex_compiler::inject_prelude_into_program(ast)
                .map_err(|e| anyhow::anyhow!("Failed to load prelude: {}", e))?;

            // Extract span map from parser
            let span_map = parser.take_span_map();

            println!("   ✅ Parsed {} successfully", filename);

            // CRITICAL: Resolve imports BEFORE borrow checker
            // This ensures imported functions are registered as valid global symbols
            let mut module_namespaces: Vec<(String, Vec<String>)> = Vec::new();
            let mut native_linker_args: Vec<String> = Vec::new();

            if !ast.imports.is_empty() {
                // Two-tier module resolution:
                // 1. vex-libs/std - Standard library packages (import "conv", "http", etc.)
                // 2. stdlib - Prelude (auto-injected core types: Vec, Box, Option, Result)
                let mut std_resolver = vex_compiler::ModuleResolver::new(PathBuf::from("vex-libs/std"));
                let mut prelude_resolver = vex_compiler::ModuleResolver::new(PathBuf::from("stdlib"));

                for import in &ast.imports {
                    let module_path = &import.module;
                    eprintln!("🔄 Resolving import: '{}'", module_path);

                    // Try standard library first (vex-libs/std), then prelude (stdlib)
                    let module_ast = match std_resolver.load_module(module_path, Some(&parser_file)) {
                        Ok(module) => {
                            eprintln!("   ✅ Loaded from vex-libs/std: {}", module_path);
                            module
                        }
                        Err(_) => {
                            eprintln!("   ⏭️  Not in vex-libs/std, trying stdlib (prelude): {}", module_path);
                            match prelude_resolver.load_module(module_path, Some(&parser_file)) {
                                Ok(module) => {
                                    eprintln!("   ✅ Loaded from stdlib: {}", module_path);
                                    module
                                }
                                Err(e) => {
                                    anyhow::bail!("⚠️  Import error for '{}': {}", module_path, e);
                                }
                            }
                        }
                    };

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
                                        vex_ast::Item::Const(const_decl) => {
                                            ast.items.push(vex_ast::Item::Const(
                                                const_decl.clone(),
                                            ));
                                        }
                                        vex_ast::Item::ExternBlock(extern_block) => {
                                            // Import extern declarations (for FFI)
                                            ast.items.push(vex_ast::Item::ExternBlock(
                                                extern_block.clone(),
                                            ));
                                        }
                                        _ => {}
                                    }
                                }
                            } else {
                                // Import only specific items
                                // BUT: always import extern blocks (for FFI dependencies)
                                for item in &module_ast.items {
                                    if let vex_ast::Item::ExternBlock(extern_block) = item {
                                        ast.items.push(vex_ast::Item::ExternBlock(
                                            extern_block.clone(),
                                        ));
                                    }
                                }

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
                                            vex_ast::Item::Const(c)
                                                if c.name == *requested =>
                                            {
                                                ast.items
                                                    .push(vex_ast::Item::Const(c.clone()));
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
                                    vex_ast::Item::Const(const_decl) => {
                                        ast.items
                                            .push(vex_ast::Item::Const(const_decl.clone()));
                                    }
                                    vex_ast::Item::ExternBlock(extern_block) => {
                                        // Import extern declarations
                                        ast.items.push(vex_ast::Item::ExternBlock(
                                            extern_block.clone(),
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                            // Save for later codegen registration
                            module_namespaces
                                .push((module_name.to_string(), imported_funcs));
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
                                    vex_ast::Item::Const(const_decl) => {
                                        ast.items
                                            .push(vex_ast::Item::Const(const_decl.clone()));
                                    }
                                    vex_ast::Item::ExternBlock(extern_block) => {
                                        // Import extern declarations
                                        ast.items.push(vex_ast::Item::ExternBlock(
                                            extern_block.clone(),
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                            // Save for later codegen registration
                            module_namespaces.push((alias.clone(), imported_funcs));
                        }
                    }

                    // Check for native dependencies in imported module's vex.json
                    // Try both vex-libs/std and stdlib paths
                    for base_path in ["vex-libs/std", "stdlib"] {
                        let module_dir = PathBuf::from(base_path).join(module_path);
                        let vex_json_path = module_dir.join("vex.json");
                        if vex_json_path.exists() {
                            if let Ok(manifest) = vex_pm::Manifest::from_file(&vex_json_path) {
                                if let Some(native_config) = manifest.get_native() {
                                    let linker = vex_pm::NativeLinker::new(&module_dir);
                                    match linker.process(native_config) {
                                        Ok(native_args_str) if !native_args_str.is_empty() => {
                                            eprintln!(
                                                "   🔗 Native libs for '{}': {}",
                                                module_path, native_args_str
                                            );
                                            // Store native args for later use
                                            for arg in native_args_str.split_whitespace() {
                                                native_linker_args.push(arg.to_string());
                                            }
                                        }
                                        Ok(_) => {} // No native args
                                        Err(e) => {
                                            eprintln!("   ⚠️  Warning: Failed to process native config for '{}': {}", module_path, e);
                                        }
                                    }
                                }
                            }
                            break; // Found vex.json, stop searching
                        }
                    }
                }
            }

            // NOW run borrow checker AFTER imports are resolved
            if !json {
                println!("   🔍 Running borrow checker...");
            }
            let mut borrow_checker = vex_compiler::BorrowChecker::new();
            if let Err(borrow_error) = borrow_checker.check_program(&mut ast) {
                // Convert borrow error to diagnostic
                let diagnostic = borrow_error.to_diagnostic();

                if json {
                    // Output as single diagnostic JSON
                    println!("{{\"diagnostics\":[{{");
                    println!("  \"level\":\"error\",");
                    println!("  \"code\":\"{}\",", diagnostic.code);
                    println!(
                        "  \"message\":\"{}\",",
                        diagnostic.message.replace('"', "\\\"")
                    );
                    println!("  \"file\":\"{}\",", diagnostic.span.file);
                    println!("  \"line\":{},", diagnostic.span.line);
                    println!("  \"column\":{}", diagnostic.span.column);
                    println!("}}]}}");
                } else {
                    eprintln!("{}", diagnostic.format(&source));
                }
                return Err(anyhow::anyhow!("Borrow check failed"));
            }
            if !json {
                println!("   ✅ Borrow check passed");
            }

            // Codegen
            let context = Context::create();
            let mut codegen = vex_compiler::ASTCodeGen::new_with_source_file(
                &context,
                &filename,
                span_map,
                &parser_file,
            );

            // Register module namespaces with codegen
            for (module_name, imported_funcs) in module_namespaces {
                codegen.register_module_namespace(module_name, imported_funcs);
            }

            let compile_result = codegen.compile_program(&ast);

            // Print diagnostics based on output format
            if codegen.has_diagnostics() {
                if json {
                    println!("{}", codegen.diagnostics().to_json());
                } else {
                    codegen.diagnostics().print_all(&source);
                    codegen.diagnostics().print_summary();
                }
            }

            // Check compilation result and diagnostics
            if let Err(e) = compile_result {
                if codegen.has_errors() {
                    return Err(anyhow::anyhow!("Compilation failed with errors"));
                } else {
                    return Err(anyhow::anyhow!("Compilation error: {}", e));
                }
            }

            if codegen.has_errors() {
                return Err(anyhow::anyhow!("Compilation failed with errors"));
            }

            // Compile to object file
            let obj_path = temp_output.with_extension("o");
            let llvm_opt_level = match opt_level {
                0 => inkwell::OptimizationLevel::None,
                1 => inkwell::OptimizationLevel::Less,
                2 => inkwell::OptimizationLevel::Default,
                3 => inkwell::OptimizationLevel::Aggressive,
                _ => inkwell::OptimizationLevel::None,
            };
            codegen
                .compile_to_object_with_opt(&obj_path, llvm_opt_level)
                .map_err(|e| anyhow::anyhow!("Object file generation error: {}", e))?;

            let mut command = Command::new("clang");
            command.arg(&obj_path).arg("-o").arg(&temp_output);

            // Add linker arguments from vex-runtime
            let linker_args = vex_runtime::get_linker_args();
            println!("  🔗 Linking with args: '{}'", linker_args);
            for arg in linker_args.split_whitespace() {
                command.arg(arg);
            }

            // Add native library linker arguments from vex.json
            if let Ok(manifest_path) = std::env::current_dir().map(|d| d.join("vex.json")) {
                if manifest_path.exists() {
                    if let Ok(manifest) = vex_pm::Manifest::from_file(&manifest_path) {
                        if let Some(native_config) = manifest.get_native() {
                            let linker = vex_pm::NativeLinker::new(std::env::current_dir()?);
                            match linker.process(native_config) {
                                Ok(native_args) if !native_args.is_empty() => {
                                    println!("  🔗 Adding native libraries: {}", native_args);
                                    for arg in native_args.split_whitespace() {
                                        command.arg(arg);
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "⚠️  Warning: Failed to process native config: {}",
                                        e
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            // Add native linker args from imported modules
            for arg in &native_linker_args {
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
            let input_str = input.to_str().unwrap_or("unknown.vx");
            let mut parser = vex_parser::Parser::new_with_file(input_str, &source)?;

            match parser.parse_file() {
                Ok(mut ast) => {
                    // ⭐ INJECT EMBEDDED PRELUDE (Layer 1 - Self-hosted)
                    ast = vex_compiler::inject_prelude_into_program(ast)
                        .map_err(|e| anyhow::anyhow!("Failed to load prelude: {}", e))?;

                    // Print warnings if any
                    let diagnostics = parser.diagnostics();
                    if !diagnostics.is_empty() {
                        for diag in diagnostics {
                            println!("{}", diag);
                        }
                    }

                    println!("✅ Syntax OK");
                    Ok(())
                }
                Err(e) => {
                    println!("❌ Parse error: {}", e);
                    anyhow::bail!(e)
                }
            }
        }
        Commands::Format { input, in_place } => {
            println!("✨ Formatting: {:?}", input);

            // Load configuration
            let config = if let Ok(cfg) = vex_formatter::Config::from_dir(
                &input.parent().unwrap_or(std::path::Path::new(".")),
            ) {
                cfg
            } else {
                vex_formatter::Config::default()
            };

            // Format the file
            let formatted = vex_formatter::format_file(&input, &config)?;

            if in_place {
                // Write back to original file
                std::fs::write(&input, &formatted)?;
                println!("✅ Formatted {} in place", input.display());
            } else {
                // Print to stdout
                println!("{}", formatted);
            }

            Ok(())
        }

        Commands::Repl { load, verbose } => {
            println!("🔮 Vex REPL v0.2.0");
            println!("Type 'exit' or Ctrl+D to quit, 'help' for commands\n");

            use std::io::{self, Write};

            let mut context_code = String::new();
            
            // Load file if provided
            if let Some(ref load_path) = load {
                match std::fs::read_to_string(load_path) {
                    Ok(content) => {
                        context_code = content.clone();
                        println!("✅ Loaded: {}", load_path.display());
                        if verbose {
                            println!("--- Context ---\n{}\n--------------", content);
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to load file: {}", e);
                    }
                }
            }

            let mut line_number = 1;
            loop {
                print!("vex [{}]> ", line_number);
                io::stdout().flush()?;

                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(0) => break, // EOF (Ctrl+D)
                    Ok(_) => {
                        let trimmed = input.trim();
                        
                        // Handle special commands
                        match trimmed {
                            "exit" | "quit" => break,
                            "help" => {
                                println!("REPL Commands:");
                                println!("  exit/quit  - Exit REPL");
                                println!("  help       - Show this help");
                                println!("  clear      - Clear context");
                                println!("  show       - Show current context");
                                println!("  load <file>- Load file into context");
                                println!("\nVex code is executed immediately");
                                continue;
                            }
                            "clear" => {
                                context_code.clear();
                                println!("✅ Context cleared");
                                continue;
                            }
                            "show" => {
                                if context_code.is_empty() {
                                    println!("(empty context)");
                                } else {
                                    println!("--- Context ---\n{}\n--------------", context_code);
                                }
                                continue;
                            }
                            _ if trimmed.starts_with("load ") => {
                                let path = trimmed.strip_prefix("load ").unwrap().trim();
                                match std::fs::read_to_string(path) {
                                    Ok(content) => {
                                        context_code = content.clone();
                                        println!("✅ Loaded: {}", path);
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Failed to load: {}", e);
                                    }
                                }
                                continue;
                            }
                            "" => continue,
                            _ => {}
                        }

                        // Build complete program with context
                        let full_code = if context_code.is_empty() {
                            // Wrap single expression in main
                            if !trimmed.contains("fn ") && !trimmed.ends_with(';') {
                                format!("fn main() {{ print({}); }}", trimmed)
                            } else {
                                trimmed.to_string()
                            }
                        } else {
                            // Add to existing context
                            format!("{}\n{}", context_code, trimmed)
                        };

                        // Try to parse and execute
                        match vex_parser::Parser::new(&full_code) {
                            Ok(mut parser) => {
                                match parser.parse() {
                                    Ok(ast) => {
                                        if verbose {
                                            println!("✓ Parsed successfully");
                                        }
                                        
                                        // For now, just parse and show success
                                        // TODO: Implement actual execution with LLVM JIT
                                        println!("✅ Code accepted (execution not yet implemented)");
                                        
                                        // Add to context if it's a declaration
                                        if trimmed.starts_with("fn ") || trimmed.starts_with("struct ") 
                                            || trimmed.starts_with("const ") {
                                            context_code.push_str("\n");
                                            context_code.push_str(trimmed);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Parse error: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ Lexer error: {}", e);
                            }
                        }

                        line_number += 1;
                    }
                    Err(e) => {
                        eprintln!("❌ Input error: {}", e);
                        break;
                    }
                }
            }

            println!("\n👋 Goodbye!");
            Ok(())
        }

        Commands::Test {
            pattern,
            verbose,
            no_parallel,
            timeout,
            bench,
            benchtime,
            count,
            benchmem,
            coverage,
            coverprofile,
            covermode,
            short,
            fuzz,
            fuzztime,
            run,
        } => {
            // Load vex.json to get test configuration
            let manifest_path = std::env::current_dir()?.join("vex.json");

            let (_test_dir, test_pattern, test_timeout, test_parallel) = if manifest_path.exists() {
                let manifest = vex_pm::Manifest::from_file(&manifest_path)?;
                let testing_config = manifest.get_testing();
                (
                    testing_config.dir,
                    testing_config.pattern,
                    testing_config.timeout,
                    testing_config.parallel,
                )
            } else {
                // Default values
                ("tests".to_string(), "**/*.test.vx".to_string(), None, true)
            };

            // Override with CLI args
            let _parallel = !no_parallel && test_parallel;
            let timeout_secs = timeout.or(test_timeout);

            // Determine test file pattern
            let search_pattern = if let Some(ref p) = pattern {
                // User provided specific pattern
                if p.ends_with(".vx") {
                    p.clone()
                } else {
                    format!("{}/**/*.test.vx", p)
                }
            } else if bench {
                "**/*.bench.vx".to_string()
            } else if fuzz.is_some() {
                "**/*fuzz*.vx".to_string()
            } else {
                test_pattern
            };

            // Discover test files
            println!("🔍 Discovering tests with pattern: {}", search_pattern);

            // Debug: print current directory
            if verbose {
                println!("   Current directory: {:?}", std::env::current_dir()?);
            }

            let test_files = discover_test_files(&search_pattern)?;

            if test_files.is_empty() {
                println!(
                    "⚠️  No test files found matching pattern: {}",
                    search_pattern
                );
                return Ok(());
            }

            println!("📋 Found {} test file(s)", test_files.len());
            if verbose {
                for file in &test_files {
                    println!("  - {}", file.display());
                }
            }

            // Execute tests
            let mut passed = 0;
            let mut failed = 0;
            let mut skipped = 0;

            println!("\n🚀 Running tests...\n");

            for test_file in &test_files {
                // Apply --run filter if specified
                if let Some(ref filter) = run {
                    let file_name = test_file.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                    if !file_name.contains(filter) {
                        continue;
                    }
                }

                // Skip if --short and test is marked slow
                if short && test_file.to_str().unwrap_or("").contains("slow") {
                    skipped += 1;
                    if verbose {
                        println!(
                            "⏭️  {} ... skipped (slow test in short mode)",
                            test_file.display()
                        );
                    }
                    continue;
                }

                // Run the test
                match run_single_test(test_file, timeout_secs, verbose) {
                    Ok(_) => {
                        passed += 1;
                        println!("✅ {} ... ok", test_file.display());
                    }
                    Err(e) => {
                        failed += 1;
                        println!("❌ {} ... FAILED", test_file.display());
                        if verbose {
                            eprintln!("   Error: {}", e);
                        }
                    }
                }
            }

            // Print summary
            println!("\n{}", "=".repeat(60));
            println!(
                "Test result: {}",
                if failed == 0 { "✅ OK" } else { "❌ FAILED" }
            );
            println!("  {} passed", passed);
            println!("  {} failed", failed);
            if skipped > 0 {
                println!("  {} skipped", skipped);
            }
            println!("{}", "=".repeat(60));

            if failed > 0 {
                std::process::exit(1);
            }

            Ok(())
        }
    }
}

// Helper function to discover test files using glob pattern
fn discover_test_files(pattern: &str) -> Result<Vec<PathBuf>> {
    let mut test_files = Vec::new();

    // Simple glob implementation for **/*.test.vx pattern
    if pattern.starts_with("**/") {
        let suffix = pattern.trim_start_matches("**/");
        walk_dir(&PathBuf::from("."), suffix, &mut test_files)?;
    } else if pattern.ends_with(".vx") {
        // Single file
        let path = PathBuf::from(pattern);
        if path.exists() {
            test_files.push(path);
        }
    } else {
        // Directory pattern
        let path = PathBuf::from(pattern);
        if path.is_dir() {
            walk_dir(&path, "*.test.vx", &mut test_files)?;
        }
    }

    Ok(test_files)
}

fn walk_dir(dir: &PathBuf, suffix: &str, results: &mut Vec<PathBuf>) -> Result<()> {
    use std::fs;

    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip hidden directories and common ignore patterns
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.')
                || name == "target"
                || name == "node_modules"
                || name == "vex-builds"
            {
                continue;
            }
        }

        if path.is_dir() {
            walk_dir(&path, suffix, results)?;
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // Match pattern: *.test.vx should match anything ending with .test.vx
            if suffix == "*.test.vx" && name.ends_with(".test.vx") {
                results.push(path);
            } else if suffix == "*.bench.vx" && name.ends_with(".bench.vx") {
                results.push(path);
            } else if name.ends_with(suffix) {
                results.push(path);
            }
        }
    }

    Ok(())
}

// Helper function to run a single test file
fn run_single_test(test_file: &PathBuf, timeout: Option<u64>, _verbose: bool) -> Result<()> {
    use std::process::Command;
    use std::time::Duration;

    // Compile and run test file (similar to vex run)
    let source = std::fs::read_to_string(test_file)?;
    let filename = test_file
        .file_stem()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid test filename"))?;

    // Create temporary executable
    let temp_output = std::env::temp_dir().join(format!("vex_test_{}", filename));

    // Parse
    let test_file_str = test_file.to_str().unwrap_or("unknown.vx");
    let mut parser = vex_parser::Parser::new_with_file(test_file_str, &source)?;
    let mut ast = parser
        .parse_file()
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    // ⭐ INJECT EMBEDDED PRELUDE (Layer 1 - Self-hosted)
    ast = vex_compiler::inject_prelude_into_program(ast)
        .map_err(|e| anyhow::anyhow!("Failed to load prelude: {}", e))?;

    let span_map = parser.take_span_map();

    // Import resolution (same as vex run)
    if !ast.imports.is_empty() {
        let mut std_resolver = vex_compiler::ModuleResolver::new(PathBuf::from("vex-libs/std"));
        let mut prelude_resolver = vex_compiler::ModuleResolver::new(PathBuf::from("stdlib"));

        for import in &ast.imports {
            let module_path = &import.module;
            
            let module_ast = match std_resolver.load_module(module_path, Some(test_file_str)) {
                Ok(module) => module,
                Err(_) => match prelude_resolver.load_module(module_path, Some(test_file_str)) {
                    Ok(module) => module,
                    Err(e) => {
                        return Err(anyhow::anyhow!("Module resolution failed: {}", e));
                    }
                }
            };

            // Merge module items into AST
            match &import.kind {
                vex_ast::ImportKind::Module | vex_ast::ImportKind::Namespace(_) => {
                    for item in &module_ast.items {
                        match item {
                            vex_ast::Item::Function(_) | vex_ast::Item::Struct(_) | 
                            vex_ast::Item::ExternBlock(_) => {
                                ast.items.push(item.clone());
                            }
                            _ => {}
                        }
                    }
                }
                vex_ast::ImportKind::Named => {
                    for requested in &import.items {
                        for item in &module_ast.items {
                            match item {
                                vex_ast::Item::Function(f) if f.name == *requested => {
                                    ast.items.push(item.clone());
                                }
                                vex_ast::Item::Struct(s) if s.name == *requested => {
                                    ast.items.push(item.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    // Compile
    let context = inkwell::context::Context::create();
    let mut codegen =
        vex_compiler::ASTCodeGen::new_with_source_file(&context, filename, span_map, test_file_str);

    codegen
        .compile_program(&ast)
        .map_err(|e| anyhow::anyhow!("Compilation error: {}", e))?;

    // Generate object file
    let obj_path = temp_output.with_extension("o");
    codegen
        .compile_to_object(&obj_path)
        .map_err(|e| anyhow::anyhow!("Object generation error: {}", e))?;

    // Link
    let mut command = Command::new("clang");
    command.arg(&obj_path).arg("-o").arg(&temp_output);

    let linker_args = vex_runtime::get_linker_args();
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

    // Execute with timeout
    let mut child = Command::new(&temp_output).spawn()?;

    let status = if let Some(timeout_secs) = timeout {
        use std::thread;
        use std::time::Instant;

        let start = Instant::now();
        let duration = Duration::from_secs(timeout_secs);

        loop {
            match child.try_wait()? {
                Some(status) => break status,
                None if start.elapsed() > duration => {
                    child.kill()?;
                    std::fs::remove_file(&obj_path).ok();
                    std::fs::remove_file(&temp_output).ok();
                    anyhow::bail!("Test timed out after {} seconds", timeout_secs);
                }
                None => thread::sleep(Duration::from_millis(100)),
            }
        }
    } else {
        child.wait()?
    };

    // Cleanup
    std::fs::remove_file(&obj_path).ok();
    std::fs::remove_file(&temp_output).ok();

    if !status.success() {
        anyhow::bail!("Test exited with code: {:?}", status.code());
    }

    Ok(())
}
