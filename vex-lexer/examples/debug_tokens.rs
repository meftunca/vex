use vex_lexer::Lexer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = if args.len() > 1 {
        &args[1]
    } else {
        "fn foo() :i32 {"
    };

    println!("Tokenizing: {}", input);
    println!();

    let mut lexer = Lexer::new(input);
    let mut count = 0;

    for result in lexer {
        count += 1;
        match result {
            Ok(token_span) => {
                println!(
                    "Token {}: {:?} at {:?}",
                    count, token_span.token, token_span.span
                );
                println!("  Text: '{}'", &input[token_span.span.clone()]);
                println!();
            }
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        }
    }

    println!("Total tokens: {}", count);
}
