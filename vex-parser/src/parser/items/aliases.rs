use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_alias(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Type, "Expected 'type'")?;

        // Capture span for the type alias name
        let span = self.token_to_diag_span(&self.peek_span().span);
        let span_id = self.span_map.generate_id();
        self.span_map.record(span_id.clone(), span);

        let name = self.consume_identifier()?;

        // Optional type parameters with bounds: type Result<T: Display, E> = ...
        let (mut type_params, _const_params) = self.parse_type_params()?; // Type aliases don't support const params

        // Optional where clause for additional bounds
        if self.match_token(&Token::Where) {
            let where_predicates = self.parse_where_clause()?;
            // Merge where clause bounds into type_params
            for predicate in where_predicates {
                match predicate {
                    WhereClausePredicate::TypeBound { type_param, bounds } => {
                        // Find matching type param and add bounds
                        if let Some(param) = type_params.iter_mut().find(|p| p.name == type_param) {
                            param.bounds.extend(bounds);
                        }
                    }
                    WhereClausePredicate::AssociatedTypeBound { .. } => {
                        // Type aliases don't support associated type bounds
                        return Err(self.make_syntax_error(
                            "Associated type bounds not supported in type aliases",
                            Some("unsupported associated type bound"),
                            Some("Use associated types in traits, not in type aliases"),
                            Some(("consider using trait with associated types", "trait T { type Item; }")),
                        ));
                    }
                }
            }
        }

        self.consume(&Token::Eq, "Expected '=' after type alias name")?;

        let ty = self.parse_type()?;

        self.consume(&Token::Semicolon, "Expected ';' after type alias")?;

        Ok(Item::TypeAlias(TypeAlias {
            is_exported: false, // Default to false
            span_id: Some(span_id),
            name,
            type_params,
            ty,
        }))
    }
}
