# FS Module - Quick Start Guide

## Installation

The FS module is part of the Vex standard library. No installation needed.

## Basic Usage

### Import

```vex
import {read_to_string, write_string} from "fs";
```

### Read File

```vex
let content: String = read_to_string("data.txt");
print(content);
```

### Write File

```vex
let text: String = "Hello, World!";
write_string("output.txt", text);
```

### Check Existence

```vex
import {exists} from "fs";

if exists("config.json") {
    // File exists
}
```

## Common Patterns

### Configuration File

```vex
import {exists, read_to_string, write_string} from "fs";

fn load_config(): String {
    let path: String = "config.json";
    
    if !exists(path) {
        // Create default config
        let default: String = "{\"version\": \"1.0\"}";
        write_string(path, default);
        return default;
    }
    
    return read_to_string(path);
}
```

### Log File

```vex
import {read_to_string, write_string, exists} from "fs";

fn append_log(message: String) {
    let path: String = "app.log";
    let existing: String = "";
    
    if exists(path) {
        existing = read_to_string(path);
    }
    
    let new_content: String = existing + message + "\n";
    write_string(path, new_content);
}
```

### Backup File

```vex
import {exists, copy} from "fs";

fn backup_file(path: String): bool {
    if !exists(path) {
        return false;
    }
    
    let backup: String = path + ".backup";
    return copy(path, backup);
}
```

## Directory Operations

```vex
import {create_dir, dir_exists, remove_dir} from "fs";

// Create directory
if !dir_exists("output") {
    create_dir("output");
}

// Check directory
if dir_exists("cache") {
    print("Cache directory exists");
}

// Remove directory
remove_dir("temp");
```

## File Management

```vex
import {copy, move_file, rename, remove} from "fs";

// Copy
copy("source.txt", "backup.txt");

// Move
move_file("temp.txt", "archive/temp.txt");

// Rename
rename("old_name.txt", "new_name.txt");

// Delete
remove("temp.txt");
```

## Error Handling

FS operations return `bool` for success/failure:

```vex
let ok: bool = write_string("output.txt", "data");

if !ok {
    print("Write failed!");
}
```

For read operations, returns empty string on failure:

```vex
let content: String = read_to_string("nonexistent.txt");

if content == "" {
    print("File not found or empty");
}
```

## Best Practices

1. **Always check existence** before operations
2. **Use absolute paths** when possible
3. **Clean up temp files** after use
4. **Handle empty string** returns from read operations
5. **Create directories** before writing files to them

## Performance Tips

- **Batch writes**: Accumulate content before writing
- **Check existence first**: Avoid unnecessary file operations
- **Use move instead of copy+remove**: More efficient
- **Prefer read_to_string**: Optimized for full file reads

## Examples

See the `tests/` directory for complete examples:

- `basic_test.vx` - Unit tests
- `demo.vx` - Basic demo
- `log_rotation.vx` - Log file rotation
- `metadata.vx` - File info checking
- `batch_processing.vx` - Process multiple files

## Limitations (v0.2.0)

- No async I/O (planned v0.4.0)
- No directory listing (planned v0.3.0)
- No file permissions (planned v0.3.0)
- No symbolic links (planned v0.3.0)
- String concatenation with numbers not yet available

---

For full API reference, see `README.md`
