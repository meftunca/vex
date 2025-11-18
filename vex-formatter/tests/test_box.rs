#[test]
fn test_format_box_vx() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("stdlib/core/src/box.vx");
    
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
    
    eprintln!("=== Formatting {} bytes ===", source.len());
    
    let config = vex_formatter::Config::default();
    match vex_formatter::format_source(&source, &config) {
        Ok(formatted) => {
            eprintln!("=== SUCCESS: {} bytes ===", formatted.len());
            assert!(!formatted.is_empty());
        }
        Err(e) => {
            panic!("Format error: {}", e);
        }
    }
}
