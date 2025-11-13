use vex_parser::Parser;
use vex_ast::*;

#[test]
fn test_operator_add_method() {
    let code = r#"
        contract Add {
            op+(other: Vec2): Vec2;
        }
    "#;
    
    let mut parser = Parser::new(code).expect("Parser::new failed");
    let program = parser.parse().expect("Parse failed");
    
    assert_eq!(program.items.len(), 1);
    
    if let Item::Contract(contract) = &program.items[0] {
        assert_eq!(contract.name, "Add");
        assert_eq!(contract.methods.len(), 1);
        
        let method = &contract.methods[0];
        assert_eq!(method.name, "op+");
        assert!(method.is_operator);
        assert_eq!(method.params.len(), 1);
        assert_eq!(method.params[0].name, "other");
    } else {
        panic!("Expected Contract, got {:?}", program.items[0]);
    }
}

#[test]
fn test_operator_impl() {
    let code = r#"
        struct Vec2 impl Add {
            x: f32,
            y: f32,
            
            op+(other: Vec2): Vec2 {
                return Vec2 { x: x + other.x, y: y + other.y };
            }
        }
    "#;
    
    let mut parser = Parser::new(code).expect("Parser::new failed");
    let program = parser.parse().expect("Parse failed");
    
    assert_eq!(program.items.len(), 1);
    
    if let Item::Struct(s) = &program.items[0] {
        assert_eq!(s.name, "Vec2");
        assert_eq!(s.impl_traits.len(), 1);
        assert_eq!(s.impl_traits[0].name, "Add");
        assert_eq!(s.methods.len(), 1);
        
        let func = &s.methods[0];
        assert_eq!(func.name, "op+");
        assert!(func.is_operator);
        assert_eq!(func.params.len(), 1);
        assert_eq!(func.params[0].name, "other");
    } else {
        panic!("Expected Struct, got {:?}", program.items[0]);
    }
}

#[test]
fn test_all_arithmetic_operators() {
    let operators = vec!["op+", "op-", "op*", "op/", "op%"];
    
    for op in operators {
        let code = format!(r#"
            contract Test {{
                fn {}(other: i32): i32;
            }}
        "#, op);
        
        let mut parser = Parser::new(&code).expect("Parser::new failed");
        let program = parser.parse().expect(&format!("Parse failed for {}", op));
        
        if let Item::Contract(contract) = &program.items[0] {
            let method = &contract.methods[0];
            assert_eq!(method.name, op);
            assert!(method.is_operator, "Expected is_operator=true for {}", op);
        } else {
            panic!("Expected Contract for {}", op);
        }
    }
}

#[test]
fn test_bitwise_operators() {
    let operators = vec!["op&", "op|", "op^", "op~", "op<<", "op>>"];
    
    for op in operators {
        let code = format!(r#"
            contract Test {{
                fn {}(other: i32): i32;
            }}
        "#, op);
        
        let mut parser = Parser::new(&code).expect("Parser::new failed");
        let program = parser.parse().expect(&format!("Parse failed for {}", op));
        
        if let Item::Contract(contract) = &program.items[0] {
            let method = &contract.methods[0];
            assert_eq!(method.name, op);
            assert!(method.is_operator, "Expected is_operator=true for {}", op);
        } else {
            panic!("Expected Contract for {}", op);
        }
    }
}

#[test]
fn test_comparison_operators() {
    let operators = vec!["op==", "op!=", "op<", "op>", "op<=", "op>="];
    
    for op in operators {
        let code = format!(r#"
            contract Test {{
                {}(other: i32): bool;
            }}
        "#, op);
        
        let mut parser = Parser::new(&code).expect("Parser::new failed");
        let program = parser.parse().expect(&format!("Parse failed for {}", op));
        
        if let Item::Contract(contract) = &program.items[0] {
            let method = &contract.methods[0];
            assert_eq!(method.name, op);
            assert!(method.is_operator, "Expected is_operator=true for {}", op);
        } else {
            panic!("Expected Contract for {}", op);
        }
    }
}

#[test]
fn test_compound_operators() {
    let operators = vec!["op+=", "op-=", "op*=", "op/=", "op%=", "op&=", "op|=", "op<<=", "op>>="];
    
    for op in operators {
        let code = format!(r#"
            contract Test {{
                {}(other: i32);
            }}
        "#, op);
        
        let mut parser = Parser::new(&code).expect("Parser::new failed");
        let program = parser.parse().expect(&format!("Parse failed for {}", op));
        
        if let Item::Contract(contract) = &program.items[0] {
            let method = &contract.methods[0];
            assert_eq!(method.name, op);
            assert!(method.is_operator, "Expected is_operator=true for {}", op);
        } else {
            panic!("Expected Contract for {}", op);
        }
    }
}

#[test]
fn test_advanced_operators() {
    let test_cases = vec![
        ("op++", "op++(): i32;"),
        ("op--", "op--(): i32;"),
        ("op**", "op**(exp: i32): i32;"),
        ("op[]", "op[](index: i32): i32;"),
        ("op[]=", "op[]=(index: i32, value: i32);"),
    ];
    
    for (op, signature) in test_cases {
        let code = format!(r#"
            contract Test {{
                {}
            }}
        "#, signature);
        
        let mut parser = Parser::new(&code).expect("Parser::new failed");
        let program = parser.parse().expect(&format!("Parse failed for {}", op));
        
        if let Item::Contract(contract) = &program.items[0] {
            let method = &contract.methods[0];
            assert_eq!(method.name, op);
            assert!(method.is_operator, "Expected is_operator=true for {}", op);
        } else {
            panic!("Expected Contract for {}", op);
        }
    }
}

#[test]
fn test_operator_with_receiver() {
    let code = r#"
        struct Vec2 impl Add {
            x: f32,
            y: f32,
            
            (self: &Vec2) op+(other: Vec2): Vec2 {
                return Vec2 { x: self.x + other.x, y: self.y + other.y };
            }
        }
    "#;
    
    let mut parser = Parser::new(code).expect("Parser::new failed");
    let program = parser.parse().expect("Parse failed");
    
    if let Item::Struct(s) = &program.items[0] {
        let func = &s.methods[0];
        assert_eq!(func.name, "op+");
        assert!(func.is_operator);
        assert!(func.receiver.is_some());
    } else {
        panic!("Expected Struct");
    }
}

#[test]
fn test_regular_function_not_operator() {
    let code = r#"
        fn add(a: i32, b: i32): i32 {
            return a + b;
        }
    "#;
    
    let mut parser = Parser::new(code).expect("Parser::new failed");
    let program = parser.parse().expect("Parse failed");
    
    if let Item::Function(func) = &program.items[0] {
        assert_eq!(func.name, "add");
        assert!(!func.is_operator);
    } else {
        panic!("Expected Function");
    }
}
