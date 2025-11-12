# Fmt Module - Vex Standard Library

**Version:** 0.1.2
**Status:** ✅ IMPLEMENTED
**Test Coverage:** Basic functionality tested

Vex'in formatting ve printing modülü. Go'nun `fmt` paketinden esinlenilmiş, Vex diline uyarlanmış API sağlar.

## 🚀 Quick Start

```vex
import { printf, sprintf, println } from "fmt";

fn main(): i32 {
    // Basit printing
    println("Hello, Vex!");

    // Formatlı printing (Go-style)
    printf("Hello, {}!", "World");

    // String formatting
    let message = sprintf("User {} logged in", "alice");

    return 0;
}
```

## 📚 API Reference

### Printing Functions

#### `print(a1: any)`, `print(a1: any, a2: any)`, `print(a1: any, a2: any, a3: any)`

Default format kullanarak stdout'a yazar. Go'nun `fmt.Print`'ine eşdeğer.

```vex
print("Hello", " ", "World");  // "Hello World"
```

#### `println()`, `println(a1: any)`, `println(a1: any, a2: any)`

Default format kullanarak stdout'a yazar ve newline ekler. Go'nun `fmt.Println`'ine eşdeğer.

```vex
println("Hello", "World");  // "Hello World\n"
println();                  // Just prints newline
```

#### `printf(format: str)`, `printf(format: str, a1: any)`, etc.

Format specifier'a göre stdout'a yazar. Go'nun `fmt.Printf`'ine eşdeğer.

```vex
printf("Name: {}, Age: {}", "Alice", 30);  // "Name: Alice, Age: 30"
printf("Hello World");                     // "Hello World"
```

### String Formatting Functions

#### `sprint(a1: any): str`

Default format kullanarak string döner. Go'nun `fmt.Sprint`'ine eşdeğer.

```vex
let s = sprint("Hello");  // s = "Hello"
```

#### `sprintf(format: str)`, `sprintf(format: str, a1: any)`, etc.

Format specifier'a göre string döner. Go'nun `fmt.Sprintf`'ine eşdeğer.

```vex
let s = sprintf("Value: {}", 42);  // s = "Value: 42"
let s = sprintf("Hello World");    // s = "Hello World"
```

### Scanning Functions

#### `scan(a1: *any)`, `scan(a1: *any, a2: *any)`, etc.

Stdin'den text okur. Go'nun `fmt.Scan`'ine eşdeğer.

```vex
let name: str;
scan(&name);
```

#### `scanln(a1: *any)`, etc.

Stdin'den text okur ve newline'da durur. Go'nun `fmt.Scanln`'ine eşdeğer.

```vex
let line: str;
scanln(&line);
```

#### `scanf(format: str, a1: *any)`, etc.

Format'a göre stdin'den okur. Go'nun `fmt.Scanf`'ine eşdeğer.

```vex
let name: str;
let age: i32;
scanf("%s %d", &name, &age);
```

### String Scanning Functions

#### `sscan(str: str, a1: *any)`, etc.

String'den text okur. Go'nun `fmt.Sscan`'ine eşdeğer.

```vex
let data = "Alice 30";
let name: str;
let age: i32;
sscan(data, &name, &age);
```

#### `sscanf(str: str, format: str, a1: *any)`, etc.

Format'a göre string'den okur. Go'nun `fmt.Sscanf`'ine eşdeğer.

```vex
let data = "Name: Alice, Age: 30";
let name: str;
let age: i32;
sscanf(data, "Name: %s, Age: %d", &name, &age);
```

## 🎯 Format Specifiers

### Placeholder Syntax

Vex fmt modülü `{}` placeholder syntax'ını kullanır:

```vex
sprintf("Hello, {}!", "World")     // "Hello, World!"
sprintf("User: {}", user.name)      // "User: alice"
sprintf("Count: {}", 42)            // "Count: 42"
```

### Multiple Arguments

```vex
sprintf("{} + {} = {}", 2, 3, 5)    // "2 + 3 = 5"
```

### Type Conversion

Otomatik type conversion ile çalışır:

```vex
let num = 42;
let pi = 3.14159;
let flag = true;

sprintf("Number: {}, Float: {}, Bool: {}", num, pi, flag);
// "Number: 42, Float: 3.14159, Bool: true"
```

## 🔧 Implementation Details

### Traits

#### `Display`

User-facing string formatting için kullanılır:

```vex
trait Display {
    fn to_string(): str;
}
```

#### `Debug`

Debugging output için kullanılır (daha verbose):

```vex
trait Debug {
    fn debug_string(): str;
}
```

### C Runtime Integration

Fmt modülü şu C runtime fonksiyonlarını kullanır:

- `vex_format_string()` - String formatting
- `vex_i32_to_string()`, `vex_f64_to_string()`, etc. - Type conversion

## 🧪 Testing

```bash
# Run fmt module tests
~/.cargo/target/debug/vex run vex-libs/std/fmt/tests/test_fmt.vx

# Run all stdlib tests
./test_stdlib_modules.sh
```

## 📝 Examples

### Basic Formatting

```vex
import { printf, sprintf } from "fmt";

fn main(): i32 {
    // Simple formatting
    printf("Hello, {}!", "World");

    // Multiple arguments
    let result = sprintf("User {} has {} messages", "alice", 5);
    println(result);

    // Type conversion
    let x = 42;
    let y = 3.14;
    printf("x = {}, y = {}", x, y);

    return 0;
}
```

### Custom Display Implementation

```vex
import { Display } from "fmt";

struct Person {
    name: str,
    age: i32,
}

impl Display for Person {
    fn to_string(): str {
        return sprintf("Person{name: {}, age: {}}", self.name, self.age);
    }
}

fn main(): i32 {
    let person = Person { name: "Alice", age: 30 };
    println(person.to_string());  // "Person{name: Alice, age: 30}"
    return 0;
}
```

### Error Handling

```vex
import { sprintf } from "fmt";

fn divide(a: i32, b: i32): Result<i32, str> {
    if b == 0 {
        return Err(sprintf("Division by zero: {} / {}", a, b));
    }
    return Ok(a / b);
}
```

## 🚧 TODO / Future Features

- [ ] Full variadic argument support
- [ ] Advanced format specifiers (`%d`, `%f`, `%s`, etc.)
- [ ] Width and precision formatting
- [ ] Custom formatters
- [ ] Color output support
- [ ] JSON formatting
- [ ] Table formatting

## 📋 Dependencies

- **vex-runtime/c/vex_string.c** - String operations
- **vex-runtime/c/vex.h** - C function declarations

## 🤝 Contributing

Fmt modülü geliştirmek için:

1. `vex-libs/std/fmt/src/lib.vx` dosyasında değişiklik yapın
2. `vex-libs/std/fmt/tests/` altına test ekleyin
3. `./test_stdlib_modules.sh` ile test edin
4. PR gönderin

---

_Part of Vex Standard Library v0.1.2_</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/vex-libs/std/fmt/README.md
