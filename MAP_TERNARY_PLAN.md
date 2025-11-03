# Map & Ternary Implementation Plan

## Status: IN PROGRESS 🔄

### 1. Map<K, V> Builtin Type ✅

#### Phase 1: Lexer ✅

- [x] Added `Map` keyword token

#### Phase 2: AST (Next)

- [ ] Add `Type::Map(Box<Type>, Box<Type>)` variant in vex-ast/src/lib.rs

#### Phase 3: Parser (Next)

- [ ] Update `parse_type_primary()` in vex-parser/src/parser/types.rs
- [ ] Syntax: `Map<string, i32>`
- [ ] Parse: `Map` + `<` + key_type + `,` + value_type + `>`

#### Phase 4: Codegen (Future)

- [ ] Map to LLVM opaque pointer or struct
- [ ] Built-in methods: `insert()`, `get()`, `remove()`, `len()`
- [ ] Consider: Link to Rust's HashMap or simple C implementation

### 2. Ternary Expression

#### Phase 1: AST (Next)

- [ ] Add `Expression::Ternary { condition, then_expr, else_expr }` in vex-ast/src/lib.rs

#### Phase 2: Parser (Next)

- [ ] Update `parse_expression()` in vex-parser/src/parser/expressions.rs
- [ ] Syntax: `condition ? then_expr : else_expr`
- [ ] Precedence: Between logical OR and assignment
- [ ] Parse after primary expression

#### Phase 3: Codegen (Next)

- [ ] Compile to if-else block in vex-compiler/src/codegen_ast/expressions/mod.rs
- [ ] Create phi node for result value

### Example Usage

```vex
// Map
let! scores = Map<string, i32>::new();
scores.insert("Alice", 100);
let score = scores.get("Alice");

// Ternary
let max = a > b ? a : b;
let status = online ? "Online" : "Offline";
```

### Implementation Order

1. ✅ Lexer: Map keyword
2. → AST: Type::Map + Expression::Ternary
3. → Parser: Map type parsing + Ternary expression parsing
4. → Codegen: Both features
5. → Tests: Examples for both

**Estimated Time**: 4-6 hours total

- Map: 2-3 hours
- Ternary: 1-2 hours
- Tests: 1 hour
