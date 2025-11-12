// Test operator method tokenization
// op+, op-, op*, op[], op++, etc.

use vex_lexer::{Lexer, Token};

#[test]
fn test_arithmetic_operator_methods() {
    let source = "op+ op- op* op/ op%";
    let mut lexer = Lexer::new(source);

    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op+".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op-".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op*".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op/".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op%".to_string())
    );
}

#[test]
fn test_bitwise_operator_methods() {
    let source = "op& op| op^ op~ op<< op>>";
    let mut lexer = Lexer::new(source);

    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op&".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op|".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op^".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op~".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op<<".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op>>".to_string())
    );
}

#[test]
fn test_comparison_operator_methods() {
    let source = "op== op!= op< op> op<= op>=";
    let mut lexer = Lexer::new(source);

    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op==".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op!=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op<".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op>".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op<=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op>=".to_string())
    );
}

#[test]
fn test_compound_assignment_operator_methods() {
    let source = "op+= op-= op*= op/= op%= op&= op|= op<<= op>>=";
    let mut lexer = Lexer::new(source);

    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op+=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op-=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op*=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op/=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op%=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op&=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op|=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op<<=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op>>=".to_string())
    );
}

#[test]
fn test_advanced_operator_methods() {
    let source = "op++ op-- op** op.. op..= op?? op[] op[]=";
    let mut lexer = Lexer::new(source);

    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op++".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op--".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op**".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op..".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op..=".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op??".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op[]".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op[]=".to_string())
    );
}

#[test]
fn test_operator_vs_identifier() {
    // Test that "op" alone is identifier, but "op+" is operator method
    let source = "op op+ operator";
    let mut lexer = Lexer::new(source);

    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::Ident("op".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op+".to_string())
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::Ident("operator".to_string())
    );
}

#[test]
fn test_operator_in_function_signature() {
    let source = "fn op+(other: Vec2): Vec2 {";
    let mut lexer = Lexer::new(source);

    assert_eq!(lexer.next().unwrap().unwrap().token, Token::Fn);
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::OperatorMethod("op+".to_string())
    );
    assert_eq!(lexer.next().unwrap().unwrap().token, Token::LParen);
    assert_eq!(
        lexer.next().unwrap().unwrap().token,
        Token::Ident("other".to_string())
    );
}
