use vex_parser::Parser;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source = if args.len() > 1 {
        args[1].as_str()
    } else {
        "fn main() { let x = 1.5e10; }"
    };

    let mut parser = Parser::new(source).unwrap();

    println!("Total tokens: {}", parser.tokens.len());
    for (i, token_span) in parser.tokens.iter().enumerate() {
        println!(
            "Token {}: {:?} at {:?}",
            i, token_span.token, token_span.span
        );
    }

    println!("\nParsing...");
    match parser.parse_file() {
        Ok(_) => println!("✅ Parse successful"),
        Err(e) => println!("❌ Parse error: {:?}", e),
    }
}
