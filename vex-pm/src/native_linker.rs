// Native C/C++ library linker
// Compiles C sources and generates linker arguments from vex.json

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::manifest::NativeConfig;

/// Native library linker
pub struct NativeLinker {
    project_root: PathBuf,
    build_dir: PathBuf,
}

impl NativeLinker {
    /// Create new native linker
    pub fn new<P: AsRef<Path>>(project_root: P) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let build_dir = project_root.join(".vex-build/native");

        Self {
            project_root,
            build_dir,
        }
    }

    /// Process native configuration and generate linker arguments
    pub fn process(&self, config: &NativeConfig) -> Result<String> {
        // Create build directory
        std::fs::create_dir_all(&self.build_dir)
            .context("Failed to create native build directory")?;

        let mut linker_args = Vec::new();

        // 1. Compile C/C++ sources
        if !config.sources.is_empty() {
            let compiled_objects = self.compile_sources(config)?;
            linker_args.extend(compiled_objects);
        }

        // 2. Add static libraries
        for static_lib in &config.static_libs {
            let lib_path = self.project_root.join(static_lib);
            if !lib_path.exists() {
                anyhow::bail!("Static library not found: {}", static_lib);
            }
            linker_args.push(lib_path.display().to_string());
        }

        // 3. Add library search paths
        for search_path in &config.search_paths {
            linker_args.push(format!("-L{}", search_path));
        }

        // 4. Add dynamic libraries (with -l prefix if needed)
        for lib in &config.libraries {
            // If library path is already complete (like "../path/libname.a"), resolve to absolute
            // Otherwise, add -l prefix for system library
            if lib.contains('/') || lib.ends_with(".a") {
                let lib_path = self.project_root.join(lib);
                if !lib_path.exists() {
                    anyhow::bail!("Library not found: {}", lib);
                }
                linker_args.push(lib_path.display().to_string());
            } else {
                linker_args.push(format!("-l{}", lib));
            }
        }

        Ok(linker_args.join(" "))
    }

    /// Compile C/C++ source files to object files
    fn compile_sources(&self, config: &NativeConfig) -> Result<Vec<String>> {
        let mut object_files = Vec::new();

        for source in &config.sources {
            let source_path = self.project_root.join(source);
            if !source_path.exists() {
                anyhow::bail!("Source file not found: {}", source);
            }

            // Determine output object file name
            let filename = source_path
                .file_stem()
                .context("Invalid source filename")?
                .to_str()
                .context("Invalid UTF-8 in filename")?;
            let obj_path = self.build_dir.join(format!("{}.o", filename));

            // Compile source to object file
            self.compile_source(&source_path, &obj_path, config)?;

            object_files.push(obj_path.display().to_string());
        }

        Ok(object_files)
    }

    /// Compile a single source file
    fn compile_source(&self, source: &Path, output: &Path, config: &NativeConfig) -> Result<()> {
        // Detect compiler (clang preferred, fallback to gcc)
        let compiler = if which::which("clang").is_ok() {
            "clang"
        } else {
            "gcc"
        };

        let mut command = Command::new(compiler);
        command
            .arg("-c")
            .arg(source)
            .arg("-o")
            .arg(output)
            .arg("-fPIC"); // Position-independent code for shared libraries

        // Add include directories
        for include_dir in &config.include_dirs {
            command.arg(format!("-I{}", include_dir));
        }

        // Add compiler flags
        for cflag in &config.cflags {
            command.arg(cflag);
        }

        println!("   🔨 Compiling: {}", source.display());

        let output_result = command.output().context("Failed to run compiler")?;

        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            anyhow::bail!("Compilation failed:\n{}", stderr);
        }

        Ok(())
    }

    /// Clean build directory
    pub fn clean(&self) -> Result<()> {
        if self.build_dir.exists() {
            std::fs::remove_dir_all(&self.build_dir)
                .context("Failed to clean native build directory")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_linker_creation() {
        let linker = NativeLinker::new("/tmp/test-project");
        assert_eq!(
            linker.build_dir,
            PathBuf::from("/tmp/test-project/.vex-build/native")
        );
    }

    #[test]
    fn test_process_libraries() {
        let config = NativeConfig {
            libraries: vec!["ssl".to_string(), "crypto".to_string()],
            search_paths: vec!["/usr/local/lib".to_string()],
            static_libs: vec![],
            sources: vec![],
            cflags: vec![],
            include_dirs: vec![],
        };

        let linker = NativeLinker::new("/tmp");
        let args = linker.process(&config).unwrap();

        assert!(args.contains("-L/usr/local/lib"));
        assert!(args.contains("-lssl"));
        assert!(args.contains("-lcrypto"));
    }
}
