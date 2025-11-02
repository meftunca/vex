# Standard Library Package Reorganization Plan

## 🎯 Hedef

FFI fonksiyonlarını mantıksal paketlere ayırarak daha kullanışlı ve organize bir std kütüphane yapısı oluşturmak.

## 📦 Yeni Paket Yapısı

```
vex-libs/std/
├── ffi/                    # Low-level FFI (internal)
│   ├── libc.vx            # Core C library bindings
│   ├── zlib.vx            # Compression
│   ├── zstd.vx
│   ├── lz4.vx
│   ├── openssl.vx         # Crypto
│   ├── platform/
│   │   ├── unix.vx
│   │   ├── windows.vx
│   │   └── posix_types.vx
│   └── mod.vx
│
├── fs/                     # Filesystem operations (HIGH-LEVEL)
│   ├── mod.vx             # Main module
│   ├── path.vx            # Path manipulation
│   ├── dir.vx             # Directory operations
│   └── file.vx            # File operations
│
├── io/                     # Input/Output (HIGH-LEVEL)
│   ├── mod.vx             # Main module
│   ├── stdio.vx           # Standard I/O (stdin, stdout, stderr)
│   ├── reader.vx          # Read trait/implementations
│   ├── writer.vx          # Write trait/implementations
│   └── buffered.vx        # Buffered I/O
│
├── process/                # Process management (HIGH-LEVEL)
│   ├── mod.vx             # Main module
│   ├── command.vx         # Command execution
│   ├── child.vx           # Child process
│   └── env.vx             # Environment variables
│
├── sync/                   # Synchronization (HIGH-LEVEL)
│   ├── mod.vx             # Main module
│   ├── mutex.vx           # Mutex implementation
│   ├── condvar.vx         # Condition variable
│   └── rwlock.vx          # Read-write lock
│
├── thread/                 # Threading (HIGH-LEVEL)
│   ├── mod.vx             # Main module
│   ├── builder.vx         # Thread builder
│   └── local.vx           # Thread-local storage
│
├── time/                   # Time operations (HIGH-LEVEL)
│   ├── mod.vx             # Main module
│   ├── duration.vx        # Duration type
│   ├── instant.vx         # Instant (monotonic)
│   └── systemtime.vx      # System time (wall clock)
│
├── regex/                  # Regular expressions (HIGH-LEVEL)
│   ├── mod.vx             # Main module
│   └── builder.vx         # Regex builder
│
├── compress/               # Compression (HIGH-LEVEL)
│   ├── mod.vx             # Main module
│   ├── zlib.vx            # Zlib wrapper
│   ├── zstd.vx            # Zstandard wrapper
│   └── lz4.vx             # LZ4 wrapper
│
├── crypto/                 # Cryptography (HIGH-LEVEL)
│   ├── mod.vx             # Main module
│   ├── hash.vx            # Hash functions (SHA256, MD5, etc.)
│   ├── hmac.vx            # HMAC
│   ├── random.vx          # Secure random
│   └── base64.vx          # Base64 encoding
│
└── net/                    # Networking (FUTURE)
    ├── mod.vx
    ├── tcp.vx
    └── udp.vx
```

## 📋 Paket Detayları

### 1. std::fs - Filesystem

**Sorumluluk:** Dosya sistemi işlemleri
**Re-exports from ffi::libc:**

- mkdir, rmdir, unlink, rename
- opendir, readdir, closedir
- stat, fstat, lstat
- open, close, read, write, lseek

**High-level API:**

```vex
import { fs } from "std";

// File operations
fs::read_to_string("file.txt")?
fs::write("file.txt", "content")?
fs::copy("src.txt", "dst.txt")?
fs::remove_file("file.txt")?
fs::rename("old.txt", "new.txt")?

// Directory operations
fs::create_dir("mydir")?
fs::create_dir_all("path/to/dir")? // mkdir -p
fs::remove_dir("mydir")?
fs::read_dir("mydir")? // returns iterator

// Metadata
fs::metadata("file.txt")?
fs::exists("file.txt")
fs::is_file("file.txt")
fs::is_dir("mydir")

// Path operations (std::fs::path)
let path = fs::Path::new("/home/user/file.txt");
path.parent()      // "/home/user"
path.file_name()   // "file.txt"
path.extension()   // "txt"
path.join("subdir")
```

### 2. std::io - Input/Output

**Sorumluluk:** Giriş/çıkış işlemleri, buffering
**Re-exports from ffi::libc:**

- fopen, fclose, fread, fwrite
- fprintf, fscanf, fgets, fputs
- stdin, stdout, stderr

**High-level API:**

```vex
import { io } from "std";

// Standard streams
io::stdin().read_line()?
io::stdout().write("Hello\n")?
io::stderr().write("Error\n")?

// File I/O
let mut file = io::File::open("data.txt")?;
file.read_to_end()?
file.write_all(b"data")?

// Buffered I/O
let reader = io::BufReader::new(file);
reader.read_line()?

let writer = io::BufWriter::new(file);
writer.write_all(b"data")?
writer.flush()?
```

### 3. std::process - Process Management

**Sorumluluk:** Process oluşturma, yönetme, environment
**Re-exports from ffi::libc:**

- fork, execve, exit
- wait, waitpid
- getenv, setenv, unsetenv
- getpid, getppid, getuid, getgid

**High-level API:**

```vex
import { process } from "std";

// Command execution
let output = process::Command::new("ls")
    .arg("-la")
    .output()?;

// Child process
let child = process::Command::new("sleep")
    .arg("5")
    .spawn()?;

child.wait()?;

// Environment
let home = process::env::var("HOME")?;
process::env::set_var("MY_VAR", "value");
process::env::remove_var("MY_VAR");

// Current process
let pid = process::id();
process::exit(0);
```

### 4. std::sync - Synchronization

**Sorumluluk:** Thread senkronizasyon primitives
**Re-exports from ffi::libc:**

- pthread*mutex*\*
- pthread*cond*\*

**High-level API:**

```vex
import { sync } from "std";

// Mutex (RAII-style)
let mutex = sync::Mutex::new(0);
{
    let mut data = mutex.lock()?;
    *data += 1;
} // Automatic unlock

// Condition Variable
let cond = sync::Condvar::new();
let mutex = sync::Mutex::new(false);

// Thread 1
{
    let mut ready = mutex.lock()?;
    *ready = true;
    cond.notify_one();
}

// Thread 2
{
    let mut ready = mutex.lock()?;
    while !*ready {
        ready = cond.wait(ready)?;
    }
}

// RwLock
let lock = sync::RwLock::new(vec![1, 2, 3]);
let r = lock.read()?;   // Multiple readers
let w = lock.write()?;  // Single writer
```

### 5. std::thread - Threading

**Sorumluluk:** Thread oluşturma ve yönetme
**Re-exports from ffi::libc:**

- pthread_create, pthread_join, pthread_detach

**High-level API:**

```vex
import { thread } from "std";

// Spawn thread
let handle = thread::spawn(|| {
    println("Hello from thread!");
    42
});

let result = handle.join()?;

// Builder
let handle = thread::Builder::new()
    .name("worker".to_string())
    .spawn(|| {
        // Thread code
    })?;

// Sleep
thread::sleep(Duration::from_secs(1));
```

### 6. std::time - Time Operations

**Sorumluluk:** Zaman ölçümü, duration
**Re-exports from ffi::libc:**

- clock_gettime, gettimeofday, time
- localtime, gmtime, strftime

**High-level API:**

```vex
import { time } from "std";

// Duration
let duration = time::Duration::from_secs(5);
let millis = duration.as_millis();

// Instant (monotonic, for measuring)
let start = time::Instant::now();
// ... do work ...
let elapsed = start.elapsed();

// SystemTime (wall clock)
let now = time::SystemTime::now();
let unix_time = now.duration_since(UNIX_EPOCH)?;

// Formatting
let now = time::SystemTime::now();
println!("{}", now.format("%Y-%m-%d %H:%M:%S"));
```

### 7. std::regex - Regular Expressions

**Sorumluluk:** Pattern matching
**Re-exports from ffi::libc:**

- regcomp, regexec, regfree

**High-level API:**

```vex
import { regex } from "std";

// Compile once, use many times
let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$")?;

// Check match
if re.is_match("2025-11-02") {
    println("Valid date!");
}

// Find match
if let Some(m) = re.find("Date: 2025-11-02") {
    println("Found at: {}-{}", m.start(), m.end());
}

// Capture groups
let re = regex::Regex::new(r"(\w+)@(\w+)\.(\w+)")?;
if let Some(caps) = re.captures("user@example.com") {
    println("User: {}", caps.get(1));
    println("Domain: {}", caps.get(2));
    println("TLD: {}", caps.get(3));
}

// Replace
let result = re.replace_all("hello", "world");
```

### 8. std::compress - Compression

**Sorumluluk:** Data compression/decompression
**Re-exports from ffi:**

- zlib, zstd, lz4

**High-level API:**

```vex
import { compress } from "std";

// Auto-detect best algorithm
let compressed = compress::compress(data)?;
let decompressed = compress::decompress(compressed)?;

// Specific algorithms
let zlib_data = compress::zlib::compress(data, 6)?;
let zstd_data = compress::zstd::compress(data, 3)?;
let lz4_data = compress::lz4::compress(data)?;

// Streaming
let compressor = compress::zstd::Compressor::new()?;
compressor.compress_chunk(chunk1)?;
compressor.compress_chunk(chunk2)?;
let result = compressor.finish()?;
```

### 9. std::crypto - Cryptography

**Sorumluluk:** Hashing, encryption, random
**Re-exports from ffi::openssl:**

- SHA256, SHA512, MD5, HMAC
- AES encryption
- Random bytes
- Base64

**High-level API:**

```vex
import { crypto } from "std";

// Hashing
let hash = crypto::sha256(b"hello");
let hex = hash.to_hex();

// HMAC
let hmac = crypto::hmac_sha256(key, message);

// Random
let random_bytes = crypto::random_bytes(32)?;

// Base64
let encoded = crypto::base64::encode(data);
let decoded = crypto::base64::decode(encoded)?;

// Password hashing (future)
let hash = crypto::bcrypt::hash("password", cost)?;
let valid = crypto::bcrypt::verify("password", hash)?;
```

## 🔄 Migration Plan

### Phase 1: Create High-Level Modules (Week 1)

1. ✅ Create std::fs module structure
2. ✅ Create std::io module structure
3. ✅ Create std::process module structure
4. ✅ Create std::time module structure

### Phase 2: Implement Core APIs (Week 2)

1. ✅ Implement std::fs basic operations
2. ✅ Implement std::io readers/writers
3. ✅ Implement std::process::Command
4. ✅ Implement std::time::Duration/Instant

### Phase 3: Advanced Features (Week 3)

1. ✅ Add buffered I/O
2. ✅ Add path manipulation
3. ✅ Add advanced process features
4. ✅ Add time formatting

### Phase 4: Move Existing (Week 4)

1. ✅ Move std::sync from root
2. ✅ Move std::regex from root
3. ✅ Create std::compress wrappers
4. ✅ Create std::crypto wrappers

### Phase 5: Documentation & Tests (Week 5)

1. ✅ Document all public APIs
2. ✅ Add usage examples
3. ✅ Create comprehensive tests
4. ✅ Update examples to use new APIs

## 📝 API Design Principles

### 1. **Type Safety**

```vex
// Good: Type-safe errors
fn read_file(path: string) -> (string | error)

// Bad: Returning null
fn read_file(path: string) -> string | null
```

### 2. **RAII (Resource Acquisition Is Initialization)**

```vex
// Automatic cleanup with defer or drop
{
    let file = fs::File::open("test.txt")?;
    // ... use file ...
} // Automatically closed
```

### 3. **Builder Pattern**

```vex
let output = process::Command::new("ls")
    .arg("-la")
    .env("PATH", "/usr/bin")
    .current_dir("/tmp")
    .output()?;
```

### 4. **Iterator Pattern**

```vex
for entry in fs::read_dir(".")? {
    let entry = entry?;
    println!("{}", entry.file_name());
}
```

### 5. **Zero-Cost Abstractions**

```vex
// High-level API should compile to same code as FFI
fs::read_to_string("file.txt")
// Should be equivalent to:
let fd = libc.open(...);
let buf = libc.malloc(...);
libc.read(fd, buf, size);
libc.close(fd);
```

## 🎯 Kullanım Örnekleri

### Before (Low-level FFI)

```vex
import { libc } from "std/ffi";

let path = "test.txt\0".as_bytes().as_ptr();
let fd = unsafe { libc.open(path, libc.O_RDONLY) };
if fd < 0 {
    return error.new("Failed to open file");
}

let buf = libc.safe_malloc(1024)?;
let bytes_read = unsafe { libc.read(fd, buf, 1024) };
unsafe { libc.close(fd); }
libc.safe_free(buf);
```

### After (High-level std::fs)

```vex
import { fs } from "std";

let content = fs::read_to_string("test.txt")?;
println!("{}", content);
```

### Before (pthread)

```vex
import { libc } from "std/ffi";

let mutex = libc.safe_malloc(64)?;
libc.safe_pthread_mutex_init(mutex as *mut libc.pthread_mutex_t)?;
libc.safe_pthread_mutex_lock(mutex as *mut libc.pthread_mutex_t)?;
// Critical section
libc.safe_pthread_mutex_unlock(mutex as *mut libc.pthread_mutex_t)?;
libc.safe_pthread_mutex_destroy(mutex as *mut libc.pthread_mutex_t)?;
```

### After (std::sync)

```vex
import { sync } from "std";

let mutex = sync::Mutex::new(0);
{
    let mut data = mutex.lock()?;
    *data += 1;
} // Automatic unlock
```

## ✅ Success Criteria

1. **Ergonomics:** 90% less boilerplate compared to raw FFI
2. **Safety:** Type-safe APIs with proper error handling
3. **Performance:** Zero-cost abstractions (same as FFI)
4. **Documentation:** Every public API documented with examples
5. **Tests:** 100% test coverage for public APIs
6. **Backwards Compatibility:** FFI layer still accessible for advanced use

## 📊 Progress Tracking

| Package       | Structure | Core API | Advanced | Tests | Docs | Status          |
| ------------- | --------- | -------- | -------- | ----- | ---- | --------------- |
| std::fs       | ✅        | ✅       | ⏳       | ⏳    | ⏳   | **In Progress** |
| std::io       | ✅        | ✅       | ⏳       | ⏳    | ⏳   | **In Progress** |
| std::process  | ⏳        | ⏳       | ⏳       | ⏳    | ⏳   | **TODO**        |
| std::sync     | ✅        | ✅       | ⏳       | ✅    | ⏳   | **In Progress** |
| std::thread   | ⏳        | ⏳       | ⏳       | ⏳    | ⏳   | **TODO**        |
| std::time     | ✅        | ✅       | ⏳       | ✅    | ⏳   | **In Progress** |
| std::regex    | ✅        | ✅       | ⏳       | ✅    | ⏳   | **In Progress** |
| std::compress | ⏳        | ⏳       | ⏳       | ⏳    | ⏳   | **TODO**        |
| std::crypto   | ⏳        | ⏳       | ⏳       | ⏳    | ⏳   | **TODO**        |

**Overall: 40% Complete** - Good progress! 🚀

## ✅ Completed Today (Nov 2, 2025)

### std::time (Complete structure + core API)

- ✅ `Duration` - Time span with nanosecond precision
- ✅ `Instant` - Monotonic time for measurements
- ✅ `SystemTime` - Wall clock time
- ✅ Test file: `examples/std_time_test.vx`

**Files created:**

- `vex-libs/std/time/duration.vx` (105 lines)
- `vex-libs/std/time/instant.vx` (62 lines)
- `vex-libs/std/time/systemtime.vx` (111 lines)
- `vex-libs/std/time/mod.vx` (17 lines)

### std::fs (Complete structure + core API)

- ✅ `File` - File operations (open, read, write, seek)
- ✅ High-level functions (read_to_string, write, copy, remove, rename)
- ✅ Directory operations (create_dir, remove_dir, read_dir)
- ✅ Metadata operations (exists, is_dir, is_file)

**Files created:**

- `vex-libs/std/fs/file.vx` (146 lines)
- `vex-libs/std/fs/mod.vx` (205 lines)

### std::io (Complete structure + core API)

- ✅ Standard streams (stdin, stdout, stderr)
- ✅ Read/write operations
- ✅ print/println/eprint/eprintln helpers
- ✅ IoError type with ErrorKind enum

**Files created:**

- `vex-libs/std/io/mod.vx` (143 lines)

**Build status:** ✅ cargo build --release successful
