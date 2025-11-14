use vex_lexer::{Lexer, Token};

#[test]
fn test_scientific_notation_basic() {
    let source = "1.5e10";
    let mut lexer = Lexer::new(source);

    let token = lexer.next().unwrap().unwrap();
    assert!(matches!(token.token, Token::FloatLiteral(_)));

    if let Token::FloatLiteral(val) = token.token {
        assert_eq!(val, 15000000000.0);
    }
}

#[test]
fn test_scientific_notation_negative_exp() {
    let source = "2.0E-5";
    let mut lexer = Lexer::new(source);

    let token = lexer.next().unwrap().unwrap();
    if let Token::FloatLiteral(val) = token.token {
        assert_eq!(val, 0.00002);
    }
}

#[test]
fn test_scientific_notation_in_statement() {
    let source = "let x = 1.5e10;";
    let tokens: Vec<_> = Lexer::new(source).map(|r| r.unwrap().token).collect();

    assert_eq!(tokens[0], Token::Let);
    assert_eq!(tokens[1], Token::Ident("x".to_string()));
    assert_eq!(tokens[2], Token::Eq);
    assert!(matches!(tokens[3], Token::FloatLiteral(_)));
    assert_eq!(tokens[4], Token::Semicolon);
}
