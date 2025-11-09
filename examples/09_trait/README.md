# Trait System Examples (Vex v1.3)

This directory contains examples demonstrating the new trait system with inline implementation.

## Syntax: `struct Foo impl Trait { ... }`

Unlike traditional Rust-style `impl Trait for Type` blocks, Vex v1.3 uses inline implementation where trait methods are defined directly inside the struct body.

## Examples

### 1. `trait_simple_test.vx` - Basic Implementation

Simple trait with single method. Demonstrates the core inline syntax.

```vex
trait Printer {
    fn print();
}

struct Message impl Printer {
    text: string,
    fn (self: &Message!) print() { }
}
```

**Status:** ✅ Compiles and runs

### 2. `trait_multiple_impl.vx` - Multiple Structs

Multiple structs implementing the same trait. Shows trait reusability.

```vex
trait Display {
    fn show();
}

struct Point impl Display {
    x: i32, y: i32,
    fn (self: &Point!) show() { }
}

struct Rectangle impl Display {
    width: i32, height: i32,
    fn (self: &Rectangle!) show() { }
}
```

**Status:** ✅ Compiles and runs

### 3. `trait_multiple_traits.vx` - Multiple Traits

Single struct implementing multiple traits with comma-separated list.

```vex
struct Person impl Display, Serializable, Comparable {
    name: string,
    age: i32,

    fn (self: &Person!) show() { }
    fn (self: &Person!) serialize() : i32 { return 1; }
    fn (self: &Person!) compare(other: &Person) : i32 { }
}
```

**Status:** ✅ Compiles and runs

### 4. `trait_system_example.vx` - Default Methods

Trait with default methods (parsed but not fully functional yet).

```vex
trait Logger {
    fn log(level: string, msg: string);
    fn info(msg: string) {  // Default method
        self.log("INFO", msg);
    }
}

struct FileLogger impl Logger {
    path: string,
    fn (self: &FileLogger!) log(level: string, msg: string) { }
}
```

**Status:** ⚠️ Parses successfully, default methods not yet inherited

## Features Supported

✅ **Inline trait implementation**: `struct Foo impl Trait { ... }`  
✅ **Multiple traits**: `struct Foo impl T1, T2, T3 { ... }`  
✅ **Method receiver syntax**: `fn (self: &Type!) method() { ... }`  
✅ **Struct-specific methods**: Mix trait methods with regular methods  
✅ **Field access in methods**: `self.field`  
✅ **Method name mangling**: `StructName_methodName`

## Features Pending

⚠️ **Default trait methods**: Not automatically inherited yet  
⚠️ **Trait inheritance**: `trait A: B, C` parsed but not implemented  
⚠️ **Trait bounds**: Generic constraints not enforced  
⚠️ **Dynamic dispatch**: Only static dispatch currently

## Running Examples

```bash
# Single example
vex compile examples/09_trait/trait_simple_test.vx
./vex-builds/trait_simple_test

# All examples
for f in examples/09_trait/trait_*.vx; do
    vex compile "$f" && ./vex-builds/$(basename "$f" .vx)
done
```

## Documentation

See `TRAIT_SYSTEM_MIGRATION_STATUS.md` for complete implementation details.
