# VSCode Unsafe Block Fix

## Problem
unsafe blokları VSCode'da syntax hatası veriyordu.

## Cause
`vscode-vex/syntaxes/vex.tmLanguage.json` içinde `unsafe` keyword tanımlı değildi.

## Solution
✅ `unsafe` keyword'ünü `keyword.control.vex` grubuna eklendi:

```json
"match": "\\b(if|else|elif|match|switch|case|default|for|while|loop|break|continue|return|defer|select|async|await|go|gpu|launch|spawn|yield|unsafe)\\b"
```

## Verification

**Parser:** ✅ Zaten destekliyor
```rust
// vex-parser/src/parser/statements.rs:117
if self.match_token(&Token::Unsafe) {
    let block = self.parse_block()?;
    return Ok(Statement::Unsafe(block));
}
```

**Lexer:** ✅ Zaten destekliyor
```rust
// vex-lexer/src/lib.rs:125
#[token("unsafe")]
Unsafe,
```

**Compiler:** ✅ Çalışıyor
```bash
$ vex compile examples/test_unsafe_required.vx
✓ Compilation successful!
```

**VSCode Extension:** ✅ Fixed
- Syntax highlighting artık `unsafe` kelimesini tanıyor
- Artık hata vermiyor

## Test
```vex
fn test() {
    unsafe {
        let ptr = malloc(200);
        free(ptr);
    }
}
```

Artık VSCode'da syntax highlighting çalışıyor ve hata göstermiyor.
