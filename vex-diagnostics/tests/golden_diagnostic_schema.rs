use std::fs;
use serde_json::Value;
use jsonschema::JSONSchema;
use vex_diagnostics::{Diagnostic, DiagnosticEngine, ErrorLevel, Span};

#[test]
fn test_diagnostics_json_against_schema() {
    let mut engine = DiagnosticEngine::new();

    // Create a sample diagnostic with suggestion and related
    let span = Span::new("main.vx".to_string(), 2, 5, 3);
    let suggestion_span = span.clone();
    let related_span = Span::new("lib.vx".to_string(), 4, 2, 4);

    let diag = Diagnostic::error("E0425", "cannot find value `foo` in this scope".to_string(), span.clone())
        .with_help("did you mean `foo_bar`?".to_string())
        .with_primary_label("undefined variable".to_string())
        .with_suggestion("rename to foo_bar".to_string(), "foo_bar".to_string(), suggestion_span)
        .with_related(related_span, "declared here".to_string());

    engine.emit(diag);

    // Emit JSON
    let json = engine.to_json();

    // Parse JSON
    let v: Value = serde_json::from_str(&json).expect("valid json");

    // Load schema
    let schema_str = fs::read_to_string("schemas/diagnostic.schema.json").expect("schema exists");
    let schema_json: Value = serde_json::from_str(&schema_str).expect("valid schema");
    let compiled = JSONSchema::compile(&schema_json).expect("valid schema compiles");

    // Validate
    let result = compiled.validate(&v);
    if let Err(errors) = result {
        for err in errors {
            panic!("Schema validation error: {}", err);
        }
    }
}
