use vex_formatter::{format_source, Config};

fn main() {
    let source = r#"
policy APIModel {
    id `json:"id"`,
    name `json:"name"`,
}

struct User with APIModel {
    id: i32,
    name: string,
}

fn main(): i32 {
    return 0;
}
"#;

    let config = Config::default();
    match format_source(source, &config) {
        Ok(formatted) => {
            println!("=== FORMATTED ===");
            println!("{}", formatted);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
