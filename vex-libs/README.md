# Vex Standard Library

Vex standard library packages (pure `.vx` files).

## Structure

```
vex-libs/
├── logger/          # Logging utilities
│   └── logger.vx
├── fs/              # Filesystem operations
│   └── fs.vx
├── time/            # Time and duration
│   └── time.vx
├── testing/         # Test framework
│   └── testing.vx
└── std/             # Core prelude
    └── prelude.vx
```

**C implementations** are in `vex-clibs/` (separate directory).

## Packages

### 1. Logger (`logger`)

- Log functions: `debug()`, `info()`, `warn()`, `error()`
- Zero-cost logging
- C runtime: `vex-clibs/vex_io.c`

### 2. Filesystem (`fs`)

- File I/O: `read_file()`, `write_file()`, `exists()`
- Path operations (planned)
- C runtime: `vex-clibs/vex_file.c`, `vex_path.c`

### 3. Time (`time`)

- Time functions: `now()`, `high_res()`
- Sleep: `sleep_ms()`
- C runtime: `vex-clibs/vex_time.c`

### 4. Testing (`testing`)

- Assertions: `assert()`, `assert_eq()`, `assert_ne()`
- Test runner (planned)
- C runtime: `vex-clibs/vex_testing.c`

## Usage in Vex

```vex
// Import standard library packages
import { logger } from "std";
import { fs } from "std";
import { time } from "std";

fn main(): i32 {
    logger.info("Starting application");

    let config = fs.read_file("config.txt");

    let start = time.high_res();
    // process_data(config);
    let elapsed = time.high_res() - start;

    return 0;
}
```

## Build Integration

**Compiler flow:**

1. Parse Vex `import { X } from "std"` statements
2. Load corresponding `.vx` package from `vex-libs/X/`
3. Generate LLVM IR with external C function declarations
4. Link `libvex.a` (compiled from `vex-clibs/`) during final compilation

**Zero-cost:** Direct C function calls in LLVM IR, no wrapper overhead.

## Platform Support

All modules are cross-platform:

- ✅ Linux
- ✅ macOS
- ✅ Windows

Platform-specific code is handled with conditional compilation (`#ifdef _WIN32`).

## Performance

- **Zero-cost abstractions** - Direct C calls from LLVM IR
- **Minimal overhead** - No vtables or dynamic dispatch
- **Optimized** - Compiled with `-O2` by default

## Adding New Packages

1. Create directory: `vex-libs/package_name/`
2. Create `package_name.vx` with Vex API
3. Add C implementation to `vex-clibs/` if needed
4. Register C functions in `vex-compiler/src/codegen_ast/builtins/stdlib.rs`
5. Update documentation

## License

MIT License - See main project LICENSE
