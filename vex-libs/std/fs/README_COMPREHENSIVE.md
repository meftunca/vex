# Vex Standard Library - File System Module

Comprehensive cross-platform file system library inspired by **Go's `os` and `path/filepath`** packages and **Rust's `std::fs`**.

## ✨ Features

### 📁 File Operations
- **Read/Write**: `read_to_string()`, `write_string()`
- **Manage**: `copy()`, `move_file()`, `rename()`, `remove()`
- **Query**: `exists()`, `path_is_file()`

### 📂 Directory Operations
- **Create**: `create_dir()`, `create_dir_all()` (like `mkdir -p`)
- **Remove**: `remove_dir()`, `remove_dir_all()` (like `rm -rf`)
- **Query**: `dir_exists()`, `path_is_dir()`

### 🛤️ Path Manipulation (Go `path/filepath` style)
- **Join/Split**: `path_join()`, `path_dir()`, `path_base()`
- **Normalize**: `path_clean()` (resolve `.` and `..`)
- **Absolute**: `path_abs()`, `path_is_absolute()`
- **Components**: `path_ext()`, `path_stem()`, `path_separator()`

### 🔍 Path Queries
- **Type Detection**: `path_is_file()`, `path_is_dir()`, `path_is_symlink()`
- **Existence**: `path_exists()`
- **Validation**: `path_is_valid()`, `path_sanitize()`
- **Comparison**: `path_equals()`, `path_starts_with()`, `path_ends_with()`

### 🔐 Permissions (Unix-style)
- **Query**: `is_readable()`, `is_writable()`, `is_executable()`
- **Manage**: `permissions()`, `set_permissions()`

### 🔗 Symbolic Links
- **Create**: `symlink_create(target, link)`
- **Read**: `symlink_read(link)`

### 🌡️ Temporary Files
- **Create**: `temp_file(prefix)`, `temp_dir(prefix)`

### 🔍 Pattern Matching
- **Glob**: `path_match(path, "*.txt")` - supports `*` and `?`

### 🗺️ Memory-Mapped Files (Zero-Copy I/O)
- **Open/Close**: `mmap_open()`, `mmap_close()`
- **Sync**: `mmap_sync()`
- **Advise**: `mmap_advise()` (hint kernel on usage pattern)
- **Allocate**: `mmap_alloc()`, `mmap_free()` (large anonymous regions)

## 🚀 Quick Start

### Basic File I/O
```vex
import {write_string, read_to_string, exists} from "fs";

fn main() {
    // Write
    write_string("/tmp/data.txt", "Hello, Vex!");
    
    // Read
    if exists("/tmp/data.txt") {
        let content = read_to_string("/tmp/data.txt");
        print(content);
    }
}
```

### Directory Operations
```vex
import {create_dir_all, dir_exists, remove_dir_all} from "fs";

fn main() {
    // Create nested directories (like mkdir -p)
    create_dir_all("/tmp/project/src/components");
    
    if dir_exists("/tmp/project") {
        print("Project directory created!\n");
    }
    
    // Cleanup
    remove_dir_all("/tmp/project");
}
```

### Path Manipulation
```vex
import {path_join, path_dir, path_base, path_ext, path_clean} from "fs";

fn main() {
    // Join paths
    let full = path_join("/tmp", "data.txt");  // "/tmp/data.txt"
    
    // Extract components
    let dir = path_dir(full);         // "/tmp"
    let file = path_base(full);       // "data.txt"
    let ext = path_ext(full);         // ".txt"
    
    // Normalize
    let clean = path_clean("./foo/../bar");  // "bar"
}
```

### Permissions
```vex
import {is_readable, permissions, set_permissions} from "fs";

fn main() {
    if is_readable("/tmp/data.txt") {
        let mode = permissions("/tmp/data.txt");
        
        // Set to rw-r--r-- (0o644)
        set_permissions("/tmp/data.txt", 0o644);
    }
}
```

### Pattern Matching
```vex
import {path_match} from "fs";

fn main() {
    if path_match("report.pdf", "*.pdf") {
        print("PDF file!\n");
    }
    
    if path_match("data.json", "data.?son") {
        print("Matches!\n");
    }
}
```

### Memory-Mapped Files
```vex
import {mmap_open, mmap_sync, mmap_close} from "fs";

fn main() {
    // Open for read-only
    let map = mmap_open("/tmp/large_file.bin", false);
    
    // Hint sequential access
    mmap_advise(map, 1);
    
    // ... use mapping ...
    
    mmap_close(map);
}
```

## 📚 API Reference

### File Operations
| Function | Description | Example |
|----------|-------------|---------|
| `read_to_string(path: str) str` | Read entire file | `read_to_string("/tmp/data.txt")` |
| `write_string(path, content: str) bool` | Write/overwrite file | `write_string("/tmp/out.txt", "data")` |
| `exists(path: str) bool` | Check existence | `exists("/tmp/file.txt")` |
| `remove(path: str) bool` | Delete file | `remove("/tmp/file.txt")` |
| `copy(src, dst: str) bool` | Copy file | `copy("/tmp/a.txt", "/tmp/b.txt")` |
| `move_file(src, dst: str) bool` | Move file | `move_file("/tmp/a", "/backup/a")` |
| `rename(old, new: str) bool` | Rename file | `rename("/tmp/old", "/tmp/new")` |

### Directory Operations
| Function | Description | Example |
|----------|-------------|---------|
| `create_dir(path: str) bool` | Create single directory | `create_dir("/tmp/mydir")` |
| `create_dir_all(path: str) bool` | Create with parents (`mkdir -p`) | `create_dir_all("/tmp/a/b/c")` |
| `remove_dir(path: str) bool` | Remove empty directory | `remove_dir("/tmp/mydir")` |
| `remove_dir_all(path: str) bool` | Remove recursively (`rm -rf`) | `remove_dir_all("/tmp/mydir")` |
| `dir_exists(path: str) bool` | Check if directory exists | `dir_exists("/tmp")` |

### Path Manipulation
| Function | Description | Example |
|----------|-------------|---------|
| `path_clean(path: str) str` | Normalize path | `path_clean("./a/../b")` → `"b"` |
| `path_join(p1, p2: str) str` | Join with OS separator | `path_join("/tmp", "file")` → `"/tmp/file"` |
| `path_dir(path: str) str` | Get parent directory | `path_dir("/tmp/file.txt")` → `"/tmp"` |
| `path_base(path: str) str` | Get filename | `path_base("/tmp/file.txt")` → `"file.txt"` |
| `path_ext(path: str) str` | Get extension | `path_ext("/tmp/file.txt")` → `".txt"` |
| `path_stem(path: str) str` | Get name without ext | `path_stem("/tmp/file.txt")` → `"file"` |
| `path_abs(path: str) str` | Convert to absolute | `path_abs("./file")` → `"/cwd/file"` |
| `path_is_absolute(path: str) bool` | Check if absolute | `path_is_absolute("/tmp")` → `true` |
| `path_separator() str` | Get OS separator | `path_separator()` → `"/"` or `"\\"` |

### Path Queries
| Function | Description | Example |
|----------|-------------|---------|
| `path_exists(path: str) bool` | Path exists (file or dir) | `path_exists("/tmp")` |
| `path_is_file(path: str) bool` | Is regular file | `path_is_file("/tmp/data.txt")` |
| `path_is_dir(path: str) bool` | Is directory | `path_is_dir("/tmp")` |
| `path_is_symlink(path: str) bool` | Is symbolic link | `path_is_symlink("/tmp/link")` |
| `path_is_valid(path: str) bool` | Valid path (no null bytes) | `path_is_valid("/tmp/file")` |
| `path_sanitize(path: str) str` | Remove invalid chars | `path_sanitize("file:bad")` → `"file_bad"` |
| `path_equals(p1, p2: str) bool` | Compare paths | `path_equals("./a", "a")` |
| `path_starts_with(path, prefix: str) bool` | Check prefix | `path_starts_with("/tmp/file", "/tmp")` |
| `path_ends_with(path, suffix: str) bool` | Check suffix | `path_ends_with("file.txt", ".txt")` |

### Permissions
| Function | Description | Example |
|----------|-------------|---------|
| `is_readable(path: str) bool` | Check read permission | `is_readable("/tmp/file")` |
| `is_writable(path: str) bool` | Check write permission | `is_writable("/tmp/file")` |
| `is_executable(path: str) bool` | Check execute permission | `is_executable("/usr/bin/ls")` |
| `permissions(path: str) u32` | Get Unix mode | `permissions("/tmp/file")` → `0o644` |
| `set_permissions(path: str, mode: u32) bool` | Set Unix mode | `set_permissions("/tmp/file", 0o755)` |

### Symlinks
| Function | Description | Example |
|----------|-------------|---------|
| `symlink_create(target, link: str) bool` | Create symlink | `symlink_create("/tmp/target", "/tmp/link")` |
| `symlink_read(link: str) str` | Read link target | `symlink_read("/tmp/link")` |

### Temp Files
| Function | Description | Example |
|----------|-------------|---------|
| `temp_file(prefix: str) str` | Create temp file | `temp_file("vex_")` → `"/tmp/vex_XXXXXX"` |
| `temp_dir(prefix: str) str` | Create temp directory | `temp_dir("build_")` → `"/tmp/build_XXXXXX"` |

### Pattern Matching
| Function | Description | Example |
|----------|-------------|---------|
| `path_match(path, pattern: str) bool` | Glob matching | `path_match("file.txt", "*.txt")` |

**Supported patterns:**
- `*` - matches any sequence of characters
- `?` - matches single character
- `[abc]` - matches any character in set
- `[a-z]` - matches any character in range

### Memory-Mapped Files
| Function | Description | Example |
|----------|-------------|---------|
| `mmap_open(path: str, writable: bool) *u8` | Open mmap | `mmap_open("/tmp/data", false)` |
| `mmap_close(mapping: *u8)` | Close mmap | `mmap_close(map)` |
| `mmap_sync(mapping: *u8) bool` | Sync to disk | `mmap_sync(map)` |
| `mmap_advise(mapping: *u8, advice: i32) bool` | Hint usage | `mmap_advise(map, 1)` |
| `mmap_alloc(size: u64) *u8` | Anon allocation | `mmap_alloc(1024*1024)` |
| `mmap_free(addr: *u8, size: u64)` | Free anon mmap | `mmap_free(mem, 1024*1024)` |

**mmap_advise() hints:**
- `0` - NORMAL (default)
- `1` - SEQUENTIAL (optimize for sequential access)
- `2` - RANDOM (optimize for random access)
- `3` - WILLNEED (prefetch pages)
- `4` - DONTNEED (release pages)

## 🏗️ Implementation

### Architecture
```
fs/
├── src/
│   ├── lib.vx                    # Current minimal API
│   └── lib_comprehensive.vx      # Full API (437 lines)
├── native/
│   └── src/
│       ├── vex_fs.c              # Basic file/dir ops (205 lines)
│       └── vex_fs.h              # Header
├── vex.json                      # Module config
└── examples/
    ├── ultra_minimal.vx          # ✅ PASSING
    ├── basic_test.vx             # String move issues
    └── comprehensive_demo.vx     # Full demo
```

### Native Dependencies
The FS module uses native C implementations from:
- `fs/native/src/vex_fs.c` - Basic file/directory operations (standalone)
- `vex-runtime/c/vex_path.c` - Path manipulation (790 lines)
- `vex-runtime/c/vex_mmap.c` - Memory-mapped files (115 lines)

### Cross-Platform Support
- **Unix/Linux**: Full support (POSIX API)
- **macOS**: Full support (BSD API)
- **Windows**: Partial support (some functions use platform-specific code)

## 🧪 Testing

### Run Tests
```bash
# From project root
cd /Users/mapletechnologies/Desktop/big_projects/vex_lang

# Ultra minimal test (✅ PASSING)
~/.cargo/target/debug/vex run vex-libs/std/fs/tests/ultra_minimal.vx

# Comprehensive demo
~/.cargo/target/debug/vex run vex-libs/std/fs/examples/comprehensive_demo.vx
```

### Test Helper Script
```bash
chmod +x vex-libs/std/fs/test.sh
./vex-libs/std/fs/test.sh
```

## 📊 Comparison with Other Languages

### Go Equivalents
| Vex FS | Go Package | Function |
|--------|------------|----------|
| `read_to_string()` | `os` | `os.ReadFile()` |
| `write_string()` | `os` | `os.WriteFile()` |
| `create_dir_all()` | `os` | `os.MkdirAll()` |
| `remove_dir_all()` | `os` | `os.RemoveAll()` |
| `path_join()` | `path/filepath` | `filepath.Join()` |
| `path_clean()` | `path/filepath` | `filepath.Clean()` |
| `path_abs()` | `path/filepath` | `filepath.Abs()` |
| `temp_file()` | `os` | `os.CreateTemp()` |
| `path_match()` | `path/filepath` | `filepath.Match()` |

### Rust Equivalents
| Vex FS | Rust Crate | Function |
|--------|------------|----------|
| `read_to_string()` | `std::fs` | `fs::read_to_string()` |
| `write_string()` | `std::fs` | `fs::write()` |
| `create_dir_all()` | `std::fs` | `fs::create_dir_all()` |
| `remove_dir_all()` | `std::fs` | `fs::remove_dir_all()` |
| `path_join()` | `std::path` | `Path::join()` |
| `permissions()` | `std::fs` | `Metadata::permissions()` |
| `symlink_create()` | `std::os::unix::fs` | `symlink()` |

## 🎯 Use Cases

### 1. Configuration Files
```vex
import {read_to_string, write_string, path_join, dir_exists, create_dir} from "fs";

fn load_config(config_dir: str) str {
    let config_file = path_join(config_dir, "app.conf");
    
    if !dir_exists(config_dir) {
        create_dir(config_dir);
        write_string(config_file, "default_config");
    }
    
    return read_to_string(config_file);
}
```

### 2. Build System
```vex
import {create_dir_all, remove_dir_all, copy, path_join} from "fs";

fn build_project() {
    // Clean old build
    remove_dir_all("./build");
    
    // Create output dirs
    create_dir_all("./build/bin");
    create_dir_all("./build/lib");
    
    // Copy artifacts
    copy("./target/app", "./build/bin/app");
}
```

### 3. Log Rotation
```vex
import {rename, remove, path_join, exists} from "fs";

fn rotate_logs(log_dir: str, max_files: i32) {
    // Shift old logs: app.3.log → app.4.log
    for i in max_files..1 {
        let old = path_join(log_dir, "app." + i.to_string() + ".log");
        let new = path_join(log_dir, "app." + (i+1).to_string() + ".log");
        
        if exists(old) {
            rename(old, new);
        }
    }
    
    // Archive current: app.log → app.1.log
    let current = path_join(log_dir, "app.log");
    let archive = path_join(log_dir, "app.1.log");
    rename(current, archive);
}
```

## 📝 Notes

- **String move semantics**: Current limitation - use string literals instead of variables in function calls to avoid borrow checker issues
- **Error handling**: Functions return `bool` for success/failure. Full `Result<T, E>` coming soon
- **Unicode**: Full UTF-8 support on Unix/Linux, limited on Windows (uses narrow APIs)
- **Performance**: Direct syscalls, zero-copy where possible, LLVM optimizations

## 🚀 Future Enhancements

- [ ] `walk()` / `walk_dir()` - Recursive directory traversal (Go `filepath.Walk`)
- [ ] `glob()` - Glob expansion returning file list (Go `filepath.Glob`)
- [ ] `list_dir()` - List directory entries (Go `os.ReadDir`)
- [ ] `metadata()` - Full file metadata struct (size, times, permissions)
- [ ] `symlink_metadata()` - Metadata without following symlinks
- [ ] `copy_dir()` - Recursive directory copy (Go `io/fs.CopyFS`)
- [ ] `file_size()` - Get file size without reading
- [ ] `file_times()` - Get access/modified/created times
- [ ] `set_times()` - Set file timestamps
- [ ] `watch()` - File system watcher (like `inotify`/`FSEvents`)

## 📄 License

BSD-3-Clause (same as Vex language)
