# 06: Error Handling Quality

**Severity:** 🟡 HIGH  
**Category:** Developer Experience / Diagnostics  
**Analysis Date:** 15 Kasım 2025  
**Status:** IDENTIFIED - USABILITY ISSUES

---

## Executive Summary

Vex'in error handling ve diagnostics sistemi **13 önemli problem** içeriyor. Error messages unclear, error recovery zayıf, IDE entegrasyonu eksik. Developer experience Rust/Go seviyesinden uzak.

**Ana Sorunlar:**
- Cryptic error messages
- No error recovery (first error stops compilation)
- No suggestions/hints
- Poor location info
- LSP diagnostics incomplete

**Impact:** Development slow, debugging hard, learning curve steep.

---

## Critical Issues (🔴)

### Issue 1: Error Recovery Missing

**File:** `vex-parser/src/lib.rs:40-58`  
**Severity:** 🔴 CRITICAL  
**Impact:** First error stops compilation, cannot see all errors

**Evidence:**
```rust
// vex-parser/src/lib.rs:40
pub fn parse(&mut self) -> Result<Program, ParseError> {
    let mut items = Vec::new();
    
    while !self.is_at_end() {
        match self.parse_item() {
            Ok(item) => items.push(item),
            Err(e) => return Err(e),  // ❌ Immediate return, no recovery
        }
    }
    
    Ok(Program { items })
}
```

**Problem:**
```vex
fn test() {
    let x = ;  // Error 1: expected expression
    let y: = 5;  // Error 2: expected type
    let z = w;  // Error 3: undefined variable
}

// Only shows Error 1, doesn't find Error 2 and 3
```

**Rust Behavior (good):**
```bash
error: expected expression, found `;`
  --> test.rs:2:13
   |
2  |     let x = ;
   |             ^ expected expression

error: expected type, found `=`
  --> test.rs:3:12
   |
3  |     let y: = 5;
   |            ^ expected type

error[E0425]: cannot find value `w` in this scope
  --> test.rs:4:13
   |
4  |     let z = w;
   |             ^ not found in this scope

error: aborting due to 3 previous errors
```

**Recommendation:**
```rust
pub fn parse(&mut self) -> Result<Program, Vec<ParseError>> {
    let mut items = Vec::new();
    let mut errors = Vec::new();
    
    while !self.is_at_end() {
        match self.parse_item() {
            Ok(item) => items.push(item),
            Err(e) => {
                errors.push(e);
                // Synchronize to next item
                self.synchronize();
            }
        }
    }
    
    if errors.is_empty() {
        Ok(Program { items })
    } else {
        Err(errors)
    }
}

fn synchronize(&mut self) {
    // Skip tokens until we find a likely item start
    while !self.is_at_end() {
        if self.peek() == Token::Fn
            || self.peek() == Token::Contract
            || self.peek() == Token::Let
        {
            return;
        }
        self.advance();
    }
}
```

**Effort:** 2-3 weeks

---

### Issue 2: Error Messages Cryptic

**File:** `vex-diagnostics/src/lib.rs`  
**Severity:** 🔴 CRITICAL  
**Impact:** Hard to understand what's wrong

**Evidence:**
```bash
$ vex run examples/test_error.vx
Error: Type mismatch
```

**Problem:**
```vex
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let x = add("5", 10);  // Type error
}

# Current error:
# Error: Type mismatch

# Should be:
# error[E0308]: mismatched types
#   --> test.vx:6:17
#    |
# 6  |     let x = add("5", 10);
#    |                 ^^^
#    |                 |
#    |                 expected `i32`, found `&str`
#    |
# help: try converting the string to an integer
#    |
# 6  |     let x = add("5".parse().unwrap(), 10);
#    |                 ++++++++++++++++++
```

**Recommendation:**
```rust
pub struct Diagnostic {
    pub code: Option<String>,  // e.g., "E0308"
    pub severity: Severity,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

pub struct Label {
    pub span: Span,
    pub message: String,
    pub style: LabelStyle,  // Primary, Secondary
}

impl Diagnostic {
    pub fn type_mismatch(
        span: Span,
        expected: &ASTType,
        found: &ASTType,
    ) -> Diagnostic {
        Diagnostic {
            code: Some("E0308".to_string()),
            severity: Severity::Error,
            message: "mismatched types".to_string(),
            labels: vec![
                Label {
                    span,
                    message: format!("expected `{}`, found `{}`", expected, found),
                    style: LabelStyle::Primary,
                },
            ],
            notes: vec![],
            help: Some(suggest_conversion(expected, found)),
        }
    }
}
```

**Effort:** 3-4 weeks

---

## High Priority Issues (🟡)

### Issue 3: No Suggestions/Hints

**File:** `vex-diagnostics/src/lib.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Users don't know how to fix errors

**Problem:**
```vex
fn main() {
    let mut x = 5;
    x = 10;  // Error: cannot assign to x
}

# Current:
# Error: cannot assign to x

# Should suggest:
# help: consider making the binding mutable
#    |
# 2  |     let mut x = 5;
#    |         +++
```

**Recommendation:**
```rust
fn suggest_fix(error: &Error) -> Option<String> {
    match error {
        Error::CannotAssign { var_name, .. } => {
            Some(format!("consider making `{}` mutable with `let! {}`", var_name, var_name))
        }
        Error::UndefinedVariable { name, similar } => {
            if let Some(suggestion) = similar {
                Some(format!("did you mean `{}`?", suggestion))
            } else {
                None
            }
        }
        // ...
    }
}
```

**Effort:** 2-3 weeks

---

### Issue 4: Poor Location Info

**File:** `vex-diagnostics/src/lib.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Hard to find error location

**Evidence:**
```bash
Error: Type mismatch
# ❌ No file, line, column
```

**Recommendation:**
```rust
pub struct Span {
    pub file: String,
    pub start: Position,
    pub end: Position,
}

pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // error[E0308]: mismatched types
        //   --> test.vx:6:17
        writeln!(f, "{}[{}]: {}", self.severity, self.code?, self.message)?;
        writeln!(f, "  --> {}:{}:{}", span.file, span.start.line, span.start.column)?;
        
        // Show source code with underline
        writeln!(f, "   |")?;
        writeln!(f, "{:3} | {}", span.start.line, source_line)?;
        writeln!(f, "   | {}^", " ".repeat(span.start.column))?;
        
        Ok(())
    }
}
```

**Effort:** 1-2 weeks

---

### Issue 5: LSP Diagnostics Incomplete

**File:** `vex-lsp/src/diagnostics.rs`  
**Severity:** 🟡 HIGH  
**Impact:** VS Code doesn't show all errors

**Evidence:**
```rust
// vex-lsp/src/diagnostics.rs
// Basic diagnostics only
pub fn publish_diagnostics(&self, uri: &Url) {
    // ❌ Missing: real-time type checking
    // ❌ Missing: borrow checker errors
    // ❌ Missing: lint warnings
}
```

**Recommendation:**
```rust
impl LanguageServer {
    fn on_document_change(&mut self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.content_changes[0].text.clone();
        
        // Reparse
        let ast = self.parser.parse(&text)?;
        
        // Type check
        let type_errors = self.type_checker.check(&ast)?;
        
        // Borrow check
        let borrow_errors = self.borrow_checker.check(&ast)?;
        
        // Lint
        let warnings = self.linter.lint(&ast)?;
        
        // Convert to LSP diagnostics
        let diagnostics = [type_errors, borrow_errors, warnings]
            .concat()
            .into_iter()
            .map(|e| to_lsp_diagnostic(e))
            .collect();
        
        self.client.publish_diagnostics(uri, diagnostics, None);
    }
}
```

**Effort:** 2-3 weeks

---

### Issue 6: No Warning System

**File:** N/A  
**Severity:** 🟡 HIGH  
**Impact:** Cannot warn about bad code

**Problem:**
```vex
fn test() {
    let x = 5;  // Unused variable
    // Should warn: unused variable `x`
}
```

**Recommendation:**
```rust
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

pub struct Linter {
    rules: Vec<Box<dyn LintRule>>,
}

trait LintRule {
    fn check(&self, ast: &Program) -> Vec<Diagnostic>;
}

struct UnusedVariableRule;
impl LintRule for UnusedVariableRule {
    fn check(&self, ast: &Program) -> Vec<Diagnostic> {
        // Find unused variables
    }
}
```

**Effort:** 2 weeks

---

### Issue 7: No Error Codes

**File:** `vex-diagnostics/src/lib.rs`  
**Severity:** 🟡 HIGH  
**Impact:** Cannot reference errors in docs

**Recommendation:**
```rust
pub enum ErrorCode {
    E0001,  // Syntax error
    E0002,  // Type mismatch
    E0308,  // Mismatched types (same as Rust)
    // ...
}

impl Diagnostic {
    pub fn new(code: ErrorCode, message: String) -> Diagnostic {
        // ...
    }
}
```

**Effort:** 1 week

---

## Medium Priority Issues (🟢)

### Issue 8: Multi-file Error Context

**Severity:** 🟢 MEDIUM  
**Impact:** Hard to understand errors spanning files

**Effort:** 2 weeks

---

### Issue 9: Error Annotations in LLVM IR

**Severity:** 🟢 MEDIUM  
**Impact:** Hard to debug codegen errors

**Effort:** 1 week

---

### Issue 10: JSON Error Output

**Severity:** 🟢 MEDIUM  
**Impact:** Cannot integrate with tools

**Recommendation:**
```bash
vex build --error-format=json
```

**Effort:** 3 days

---

### Issue 11: Colored Output

**Severity:** 🟢 MEDIUM  
**Impact:** Errors harder to read

**Effort:** 3 days

---

## Low Priority Issues (🔵)

### Issue 12: Error Explanations

**Severity:** 🔵 LOW  
**Impact:** No `--explain` flag like Rust

**Recommendation:**
```bash
$ vex --explain E0308
```

**Effort:** 2 weeks

---

### Issue 13: Error Statistics

**Severity:** 🔵 LOW  
**Impact:** Cannot track error trends

**Effort:** 1 week

---

## Metrics Summary

| Category | Critical | High | Medium | Low | Total |
|----------|----------|------|--------|-----|-------|
| Parser Errors | 1 | 0 | 1 | 0 | 2 |
| Error Messages | 1 | 3 | 1 | 1 | 6 |
| LSP Integration | 0 | 1 | 0 | 0 | 1 |
| Warning System | 0 | 1 | 0 | 0 | 1 |
| Tooling | 0 | 1 | 2 | 1 | 4 |
| **TOTAL** | **2** | **6** | **4** | **2** | **13** |

---

## Implementation Roadmap

### Phase 1: Critical Fixes (Week 1-2)
- [ ] Implement error recovery
- [ ] Improve error messages (format, context)

### Phase 2: High Priority (Week 3-6)
- [ ] Add suggestions/hints
- [ ] Fix location info
- [ ] Improve LSP diagnostics
- [ ] Add warning system
- [ ] Add error codes

### Phase 3: Medium Priority (Week 7-9)
- [ ] Multi-file error context
- [ ] JSON error output
- [ ] Colored output

### Phase 4: Low Priority (Week 10-11)
- [ ] Error explanations
- [ ] Statistics

---

## Testing Plan

```vex
// test_error_recovery.vx
fn test() {
    let x = ;  // Error 1
    let y: = 5;  // Error 2
    let z = w;  // Error 3
}
// Should report all 3 errors

// test_suggestions.vx
fn main() {
    let x = 5;
    x = 10;  // Should suggest: let! x
}

// test_unused.vx
fn test() {
    let x = 5;  // Should warn: unused variable
}
```

```bash
# Test error messages
vex build test_error.vx 2>&1 | grep "expected \`i32\`, found \`&str\`"

# Test JSON output
vex build --error-format=json test_error.vx | jq '.errors'

# Test LSP
# Open in VS Code, should show squiggly lines
```

---

## Related Issues

- [01_TYPE_SYSTEM_GAPS.md](./01_TYPE_SYSTEM_GAPS.md) - Better type errors needed
- [02_BORROW_CHECKER_WEAKNESSES.md](./02_BORROW_CHECKER_WEAKNESSES.md) - Lifetime errors cryptic

---

## References

- Rust error messages: https://doc.rust-lang.org/error-index.html
- Elm compiler errors: https://elm-lang.org/news/compiler-errors-for-humans
- Ariadne (error reporting crate): https://github.com/zesterer/ariadne

---

**Next Steps:** Implement error recovery first, then improve message format.
