use crate::parser::Parser;
use crate::ParseError;
use vex_ast::{Policy, PolicyField};
use vex_lexer::Token;

impl<'a> Parser<'a> {
    /// Parse policy declaration: policy APIModel { id `json:"id"`, name `json:"name"` }
    /// Grammar:
    ///   policy_decl := 'policy' IDENT ('with' policy_list)? '{' policy_field_list '}'
    ///   policy_list := IDENT (',' IDENT)*
    ///   policy_field_list := policy_field (',' policy_field)* ','?
    ///   policy_field := IDENT TAG_LITERAL
    pub fn parse_policy(&mut self) -> Result<Policy, ParseError> {
        // Expect 'policy' keyword
        self.consume(&Token::Policy, "Expected 'policy' keyword")?;

        // Parse policy name
        let name = self.consume_identifier()?;

        // Optional: with clause for policy composition
        let mut parent_policies = Vec::new();
        if self.check(&Token::With) {
            self.advance(); // consume 'with'

            // Parse comma-separated parent policy names
            loop {
                parent_policies.push(self.consume_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        // Expect opening brace
        self.consume(&Token::LBrace, "Expected '{' after policy name")?;

        // Parse policy fields
        let mut fields = Vec::new();
        let mut steps = 0usize;
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.guard_tick(&mut steps, "policy body parse timeout", Self::PARSE_LOOP_DEFAULT_MAX_STEPS) {
                break;
            }
            let field = self.parse_policy_field()?;
            fields.push(field);

            // Optional comma
            if self.check(&Token::Comma) {
                self.advance();
            }
        }

        // Expect closing brace
        self.consume(&Token::RBrace, "Expected '}' after policy fields")?;

        Ok(Policy {
            is_exported: false, // Default to false
            name,
            parent_policies,
            fields,
        })
    }

    /// Parse a single policy field: field_name `metadata`
    fn parse_policy_field(&mut self) -> Result<PolicyField, ParseError> {
        // Parse field name
        let name = self.consume_identifier()?;

        // Expect backtick metadata - check current token
        match self.peek() {
            Token::Tag(tag) => {
                let metadata = tag.clone();
                self.advance();
                Ok(PolicyField { name, metadata })
            }
            _ => Err(self.make_syntax_error(
                "Expected backtick metadata (e.g., `json:\"id\"`)",
                Some("expected backtick metadata"),
                Some("Provide metadata in backticks after the field name, e.g., `json:\"id\"`"),
                Some(("try metadata", "`json:\"id\"`")),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn test_parse_simple_policy() {
        let input = r#"
            policy APIModel {
                id `json:"id"`,
                name `json:"name"`
            }
        "#;

        let mut parser = Parser::new(input).expect("Failed to create parser");
        let policy = parser.parse_policy().expect("Failed to parse policy");

        assert_eq!(policy.name, "APIModel");
        assert_eq!(policy.parent_policies.len(), 0);
        assert_eq!(policy.fields.len(), 2);
        assert_eq!(policy.fields[0].name, "id");
        assert_eq!(policy.fields[0].metadata, r#"json:"id""#);
        assert_eq!(policy.fields[1].name, "name");
        assert_eq!(policy.fields[1].metadata, r#"json:"name""#);
    }

    #[test]
    fn test_parse_policy_with_composition() {
        let input = r#"
            policy Child with Parent1, Parent2 {
                id `json:"id"`
            }
        "#;

        let mut parser = Parser::new(input).expect("Failed to create parser");
        let policy = parser.parse_policy().expect("Failed to parse policy");

        assert_eq!(policy.name, "Child");
        assert_eq!(policy.parent_policies, vec!["Parent1", "Parent2"]);
        assert_eq!(policy.fields.len(), 1);
    }

    #[test]
    fn test_parse_policy_with_complex_metadata() {
        let input = r#"
            policy UserModel {
                id `json:"id" db:"user_id" validate:"required"`,
                email `json:"email" validate:"email,required"`
            }
        "#;

        let mut parser = Parser::new(input).expect("Failed to create parser");
        let policy = parser.parse_policy().expect("Failed to parse policy");

        assert_eq!(
            policy.fields[0].metadata,
            r#"json:"id" db:"user_id" validate:"required""#
        );
        assert_eq!(
            policy.fields[1].metadata,
            r#"json:"email" validate:"email,required""#
        );
    }
}
