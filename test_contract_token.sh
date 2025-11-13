#!/bin/bash
# Quick test for contract token
cd "$(dirname "$0")"

echo "Testing contract keyword in lexer..."
cargo test -q --package vex-lexer test_contract 2>&1 | grep -E "(test_contract|passed|failed)"

if [ $? -eq 0 ]; then
    echo "✅ Contract keyword test exists"
else
    echo "⚠️  Adding contract keyword test..."
    
    # Add test to vex-lexer/src/lib.rs
    cat >> vex-lexer/src/lib.rs << 'EOF'

    #[test]
    fn test_contract_keyword() {
        let source = "contract Add { }";
        let mut lexer = Lexer::new(source);
        
        let first = lexer.next().unwrap().unwrap().token;
        assert_eq!(first, Token::Contract, "Expected Contract token, got {:?}", first);
    }
EOF
    
    cargo test -q --package vex-lexer test_contract_keyword
fi
