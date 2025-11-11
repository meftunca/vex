# Vex FS Module

High-performance, safe file system operations for Vex.

## Features

- ✅ File I/O (read, write, append)
- ✅ Directory operations (create, remove, exists)
- ✅ File management (copy, move, rename, remove)
- ✅ String-based API (UTF-8 safe)
- ✅ C runtime integration (zero-overhead)

## API Reference

### File Operations

```vex
import {read_to_string, write_string, exists} from "fs";

// Read entire file
let content: String = read_to_string("data.txt");

// Write file
let ok: bool = write_string("output.txt", "Hello!");

// Check existence
if exists("config.json") {
    // ...
}
```

### File Handles

```vex
import {open, create, append, close} from "fs";

// Open for reading
let f: File = open("input.txt");
// ... read operations
close(&f);

// Create for writing
let f2: File = create("output.txt");
// ... write operations
close(&f2);

// Append mode
let f3: File = append("log.txt");
// ... append operations
close(&f3);
```

### File Management

```vex
import {copy, move_file, rename, remove} from "fs";

// Copy file
copy("source.txt", "backup.txt");

// Move file
move_file("temp.txt", "archive/temp.txt");

// Rename
rename("old.txt", "new.txt");

// Delete
remove("temp.txt");
```

### Directory Operations

```vex
import {create_dir, remove_dir, dir_exists} from "fs";

// Create directory
create_dir("output");

// Check existence
if dir_exists("cache") {
    // ...
}

// Remove directory
remove_dir("temp");
```

## Examples

### Read Configuration File

```vex
import {read_to_string, exists} from "fs";

fn load_config(): String {
    let path: String = "config.json";
    
    if !exists(path) {
        return "{}"; // Default empty config
    }
    
    return read_to_string(path);
}
```

### Write Log File

```vex
import {append, close, write_string} from "fs";

fn log_message(msg: String) {
    let path: String = "app.log";
    let timestamp: String = "[2025-11-11] ";
    let line: String = timestamp + msg + "\n";
    
    write_string(path, line);
}
```

### Backup Files

```vex
import {copy, exists} from "fs";

fn backup_file(path: String): bool {
    if !exists(path) {
        return false;
    }
    
    let backup_path: String = path + ".backup";
    return copy(path, backup_path);
}
```

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| read_to_string | ✅ | Full UTF-8 support |
| write_string | ✅ | Atomic writes |
| File handles | ✅ | RAII cleanup |
| copy/move | ✅ | Platform-optimized |
| Directories | ✅ | Recursive support planned |
| Permissions | ⏳ | Planned for v0.3.0 |
| Async I/O | ⏳ | Planned for v0.4.0 |
| Memory mapping | ⏳ | Planned for v0.5.0 |

## Performance

- **Zero-copy reads**: Direct buffer access
- **Buffered writes**: Automatic flushing
- **Native integration**: C runtime (libc)
- **LLVM optimizations**: Inlined hot paths

## Testing

```bash
cd vex-libs/std/fs
./tests/run_tests.sh
```

## Safety

- **Path validation**: Prevents directory traversal
- **Error handling**: Graceful failures (returns empty string/false)
- **Resource cleanup**: Automatic file handle closing
- **Buffer safety**: No overflow vulnerabilities

## Version History

- **v0.2.0** (2025-11-11): Enhanced API, directory operations
- **v0.1.0** (2025-11-10): Initial release, basic file I/O

---

**Author**: Vex Team  
**License**: MIT
