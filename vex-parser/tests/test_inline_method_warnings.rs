// Test that inline struct methods generate deprecation warnings

use vex_parser::Parser;

#[test]
fn test_inline_method_generates_warning() {
    let code = r#"
        struct Vec2 {
            x: f32,
            y: f32,
            
            add(other: Vec2): Vec2 {
                return Vec2 { x: self.x + other.x, y: self.y + other.y };
            }
        }
    "#;
    
    let mut parser = Parser::new(code).expect("Parser::new failed");
    let _program = parser.parse().expect("Parse failed");
    
    // Check that we got a warning
    let diagnostics = parser.diagnostics();
    assert_eq!(diagnostics.len(), 1, "Expected 1 warning");
    
    let warning = &diagnostics[0];
    assert_eq!(warning.level, vex_diagnostics::ErrorLevel::Warning);
    assert_eq!(warning.code, "W0001");
    assert!(warning.message.contains("deprecated"));
    assert!(warning.help.is_some());
}

#[test]
fn test_operator_method_generates_warning() {
    let code = r#"
        struct Vec2 impl Add {
            x: f32,
            y: f32,
            
            op+(other: Vec2): Vec2 {
                return Vec2 { x: self.x + other.x, y: self.y + other.y };
            }
        }
    "#;
    
    let mut parser = Parser::new(code).expect("Parser::new failed");
    let _program = parser.parse().expect("Parse failed");
    
    // Check that we got a warning
    let diagnostics = parser.diagnostics();
    assert_eq!(diagnostics.len(), 1, "Expected 1 warning");
    
    let warning = &diagnostics[0];
    assert_eq!(warning.level, vex_diagnostics::ErrorLevel::Warning);
    assert_eq!(warning.code, "W0001");
    assert!(warning.message.contains("deprecated"));
}

#[test]
fn test_receiver_method_generates_warning() {
    let code = r#"
        struct Point {
            x: i32,
            y: i32,
            
            (self: &Point) display(): String {
                return "point";
            }
        }
    "#;
    
    let mut parser = Parser::new(code).expect("Parser::new failed");
    let _program = parser.parse().expect("Parse failed");
    
    // Check that we got a warning
    let diagnostics = parser.diagnostics();
    assert_eq!(diagnostics.len(), 1, "Expected 1 warning");
    
    let warning = &diagnostics[0];
    assert_eq!(warning.level, vex_diagnostics::ErrorLevel::Warning);
    assert_eq!(warning.code, "W0001");
}

#[test]
fn test_external_method_no_warning() {
    let code = r#"
        struct Vec2 {
            x: f32,
            y: f32,
        }
        
        fn (self: &Vec2) add(other: Vec2): Vec2 {
            return Vec2 { x: self.x + other.x, y: self.y + other.y };
        }
    "#;
    
    let mut parser = Parser::new(code).expect("Parser::new failed");
    let _program = parser.parse().expect("Parse failed");
    
    // External methods should NOT generate warnings
    let diagnostics = parser.diagnostics();
    assert_eq!(diagnostics.len(), 0, "Expected no warnings for external methods");
}
