/// Build script for vex-compiler
/// 
/// Ensures embedded prelude files exist and are valid at compile time.
/// This prevents compilation if prelude files are missing or empty.

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/prelude/");

    // Verify all prelude files exist and are not empty
    let prelude_files = [
        "src/prelude/lib.vx",
        "src/prelude/vec.vx",
        "src/prelude/option.vx",
        "src/prelude/result.vx",
        "src/prelude/box.vx",
        "src/prelude/ops.vx",
        "src/prelude/builtin_contracts.vx",
    ];

    for file in &prelude_files {
        let path = Path::new(file);
        
        if !path.exists() {
            panic!(
                "❌ PRELUDE FILE MISSING: {}\n\
                 The compiler requires all Layer 1 prelude files to be present.\n\
                 Expected location: vex-compiler/{}",
                file, file
            );
        }

        let metadata = std::fs::metadata(path)
            .unwrap_or_else(|e| panic!("Failed to read metadata for {}: {}", file, e));

        if metadata.len() == 0 {
            panic!(
                "❌ PRELUDE FILE EMPTY: {}\n\
                 The compiler cannot use an empty prelude file.\n\
                 Please ensure all prelude modules have valid Vex code.",
                file
            );
        }

        println!("   ✅ Verified prelude file: {} ({} bytes)", file, metadata.len());
    }

    println!("✅ All prelude files verified - Layer 1 embedding ready");
}
