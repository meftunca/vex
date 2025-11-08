// CLI commands for package manager

use crate::manifest::Manifest;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Create a new Vex project
pub fn create_new_project(name: &str, path: Option<PathBuf>) -> Result<PathBuf> {
    let project_path = path.unwrap_or_else(|| PathBuf::from(name));

    // Check if directory already exists
    if project_path.exists() {
        anyhow::bail!("Directory already exists: {}", project_path.display());
    }

    // Create project structure
    create_project_structure(&project_path, name)?;

    println!("✅ Created new Vex project: {}", name);
    println!("   Path: {}", project_path.display());
    println!("\nNext steps:");
    println!("   cd {}", name);
    println!("   vex build");
    println!("   vex run");

    Ok(project_path)
}

/// Initialize vex.json in existing directory
pub fn init_project(path: Option<PathBuf>) -> Result<PathBuf> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    // Check if vex.json already exists
    let manifest_path = project_path.join("vex.json");
    if manifest_path.exists() {
        anyhow::bail!("vex.json already exists in {}", project_path.display());
    }

    // Get project name from directory
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project");

    // Create manifest
    let mut manifest = Manifest::default();
    manifest.name = project_name.to_string();

    // Write vex.json
    manifest.to_file(&manifest_path)?;

    println!("✅ Initialized vex.json in {}", project_path.display());

    Ok(project_path)
}

/// Create project directory structure
fn create_project_structure(path: &Path, name: &str) -> Result<()> {
    // Create directories
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory: {}", path.display()))?;

    let src_dir = path.join("src");
    let tests_dir = path.join("tests");

    fs::create_dir(&src_dir).with_context(|| format!("Failed to create src directory"))?;

    fs::create_dir(&tests_dir).with_context(|| format!("Failed to create tests directory"))?;

    // Create vex.json
    let mut manifest = Manifest::default();
    manifest.name = name.to_string();
    manifest.to_file(path.join("vex.json"))?;

    // Create src/lib.vx
    let lib_content = format!(
        r#"// {} - Main library file

export fn greet(name: string): string {{
    return "Hello, " + name + "!";
}}

fn main(): i32 {{
    let message = greet("Vex");
    println(message);
    return 0;
}}
"#,
        name
    );

    fs::write(src_dir.join("lib.vx"), lib_content).context("Failed to create src/lib.vx")?;

    // Create tests/lib_test.vx
    let test_content = r#"// Tests for library

import { greet } from "../src/lib";

fn test_greet(): i32 {
    let result = greet("World");
    
    if result != "Hello, World!" {
        println("❌ Test failed: expected 'Hello, World!', got:", result);
        return 1;
    }
    
    println("✅ test_greet passed");
    return 0;
}

fn main(): i32 {
    return test_greet();
}
"#;

    fs::write(tests_dir.join("lib_test.vx"), test_content)
        .context("Failed to create tests/lib_test.vx")?;

    // Create .gitignore
    let gitignore_content = r#"# Vex build artifacts
vex-builds/
*.ll
*.o
*.a

# Dependency cache
.vex/

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db
"#;

    fs::write(path.join(".gitignore"), gitignore_content).context("Failed to create .gitignore")?;

    // Create README.md
    let readme_content = format!(
        r#"# {}

A Vex project.

## Usage

```bash
# Build
vex build

# Run
vex run

# Test
vex test
```

## License

MIT
"#,
        name
    );

    fs::write(path.join("README.md"), readme_content).context("Failed to create README.md")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_create_project() {
        let temp_dir = env::temp_dir();
        let project_name = "test_vex_project";
        let project_path = temp_dir.join(project_name);

        // Clean up if exists
        if project_path.exists() {
            fs::remove_dir_all(&project_path).ok();
        }

        // Create project
        let result = create_new_project(project_name, Some(project_path.clone()));
        assert!(result.is_ok());

        // Verify structure
        assert!(project_path.join("vex.json").exists());
        assert!(project_path.join("src/lib.vx").exists());
        assert!(project_path.join("tests/lib_test.vx").exists());
        assert!(project_path.join(".gitignore").exists());
        assert!(project_path.join("README.md").exists());

        // Clean up
        fs::remove_dir_all(&project_path).ok();
    }
}
