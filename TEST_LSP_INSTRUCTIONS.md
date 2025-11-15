# Testing Real-Time LSP Diagnostics

## Setup

1. **Install Vex VS Code Extension**
   ```bash
   cd vscode-vex
   npm install
   npm run compile
   code --install-extension .
   ```

2. **Build LSP Server**
   ```bash
   cargo build --bin vex-lsp
   ```

3. **Configure VS Code**
   - Open VS Code settings (Cmd+,)
   - Search for "Vex Language Server"
   - Ensure path is: `~/.cargo/target/debug/vex-lsp`

## Test Cases

### 1. Unused Variable Warning (W0001)
Open `examples/test_lsp_realtime.vx`:
- `unused_x` should show **yellow squiggle** (warning)
- Hover should show: "warning[W0001]: unused variable: `unused_x`"
- `_ignored` should NOT show warning (underscore prefix)
- `y` should NOT show warning (used in print)

### 2. Unused Parameter Warning (W0001)
In `test_func`:
- `unused_param` should show **yellow squiggle**
- Message: "warning[W0001]: unused variable: `unused_param`"

### 3. Real-Time Updates
Type new code:
```vex
let fresh_unused = 100;
```
- Warning should appear **immediately** (no save required)
- Use the variable: `print(fresh_unused);`
- Warning should **disappear immediately**

### 4. Borrow Checker Errors
Add this code:
```vex
let x = 10;
x = 20;  // Error: cannot assign to immutable
```
- Should show **red squiggle**
- Error: "error[E0594]: cannot assign to immutable variable `x`"
- Help text: "consider making this binding mutable: `let! x`"

### 5. Multiple Diagnostics
Mix errors and warnings:
```vex
let unused = 1;     // Warning
let y = 2;
y = 3;             // Error (immutable)
let _ok = 4;       // No warning
```
Should show:
- Line 1: Yellow squiggle (unused variable)
- Line 3: Red squiggle (borrow checker error)
- Line 4: No squiggle

## Verification Checklist

- [ ] Warnings show yellow squiggly underlines
- [ ] Errors show red squiggly underlines
- [ ] Hover shows full diagnostic message + code
- [ ] Diagnostics update in real-time (no save needed)
- [ ] Underscore-prefixed variables ignored
- [ ] Multiple diagnostics work simultaneously
- [ ] VS Code Problems panel shows all issues
- [ ] Code actions available (if implemented)

## Expected Architecture

```
User types code → didChange event
  ↓
LSP Server (vex-lsp)
  ↓
parse_and_diagnose()
  ├─ Parser (parse_with_recovery) → syntax errors
  ├─ Linter → unused variable warnings
  └─ BorrowChecker → ownership errors
  ↓
Publish diagnostics to VS Code
  ↓
VS Code shows squiggles + Problems panel
```

## Troubleshooting

**No diagnostics appearing:**
- Check LSP server is running: `ps aux | grep vex-lsp`
- Check VS Code Output panel: "Vex Language Server"
- Restart VS Code
- Check file extension is `.vx`

**Old errors not clearing:**
- Save file (Cmd+S)
- Close and reopen file
- Restart LSP server

**Performance issues:**
- LSP re-parses on every keystroke
- Large files may lag
- Check CPU usage: `top | grep vex-lsp`

## Success Criteria

✅ **Rust/Go-level error quality achieved when:**
1. All warnings show immediately (no compilation needed)
2. Error messages are clear and actionable
3. Multiple error types work together (syntax, borrow, lint)
4. Help text guides users to fix issues
5. Performance is acceptable (<100ms for typical files)
