# Vex Stdlib Planning - 06: System and OS

**Priority:** 6
**Status:** Partial (fs exists, others missing)
**Dependencies:** builtin, unsafe, io

## 📦 Packages in This Category

### 6.1 os
**Status:** ❌ Missing (critical for system interaction)
**Description:** Operating system interface

#### Required Types
```vex
struct File {
    fd: i32,
    name: str,
    // internal fields
}

struct FileInfo {
    name: str,
    size: i64,
    mode: FileMode,
    mod_time: time.Time,
    is_dir: bool,
}

struct FileMode {
    perm: u32,
}

struct ProcAttr {
    dir: str,
    env: []str,
    files: []*File,
    sys: *SysProcAttr,
}
```

#### Required Functions
```vex
// File operations
fn create(name: str): Result<File, Error>
fn open(name: str): Result<File, Error>
fn open_file(name: str, flag: int, perm: FileMode): Result<File, Error>
fn stat(name: str): Result<FileInfo, Error>
fn lstat(name: str): Result<FileInfo, Error>

// Directory operations
fn mkdir(name: str, perm: FileMode): Result<(), Error>
fn mkdir_all(path: str, perm: FileMode): Result<(), Error>
fn remove(name: str): Result<(), Error>
fn remove_all(path: str): Result<(), Error>
fn rename(oldpath: str, newpath: str): Result<(), Error>

// Working directory
fn getwd(): Result<str, Error>
fn chdir(dir: str): Result<(), Error>

// Environment
fn getenv(key: str): str
fn setenv(key: str, value: str): Result<(), Error>
fn unsetenv(key: str): Result<(), Error>
fn environ(): []str

// Process operations
fn find_process(pid: int): Result<Process, Error>
fn start_process(name: str, argv: []str, attr: *ProcAttr): Result<Process, Error>
fn exit(code: int) -> !
fn getpid(): int
fn getppid(): int

// Signals
fn signal_notify(c: chan<Signal>, sig: ...Signal)
fn signal_stop(c: chan<Signal>)
fn signal_ignore(sig: ...Signal)
fn signal_reset(sig: ...Signal)
```

#### Dependencies
- builtin
- io
- time
- syscall

### 6.2 syscall
**Status:** ❌ Missing (low-level system access)
**Description:** Low-level system calls

#### Required Functions
```vex
// File operations
fn open(path: *u8, flags: int, mode: u32): Result<i32, Error>
fn close(fd: i32): Result<(), Error>
fn read(fd: i32, buf: []u8): Result<usize, Error>
fn write(fd: i32, buf: []u8): Result<usize, Error>
fn lseek(fd: i32, offset: i64, whence: int): Result<i64, Error>

// Process operations
fn fork(): Result<i32, Error>
fn execve(path: *u8, argv: []*u8, envp: []*u8): Result<(), Error>
fn wait4(pid: i32, wstatus: *i32, options: int, rusage: *Rusage): Result<i32, Error>
fn getpid(): i32
fn getppid(): i32
fn kill(pid: i32, sig: i32): Result<(), Error>

// Memory operations
fn mmap(addr: *u8, length: usize, prot: int, flags: int, fd: i32, offset: i64): Result<*u8, Error>
fn munmap(addr: *u8, length: usize): Result<(), Error>

// Network operations
fn socket(domain: int, typ: int, protocol: int): Result<i32, Error>
fn bind(sockfd: i32, addr: *Sockaddr, addrlen: u32): Result<(), Error>
fn listen(sockfd: i32, backlog: int): Result<(), Error>
fn accept(sockfd: i32, addr: *Sockaddr, addrlen: *u32): Result<i32, Error>
fn connect(sockfd: i32, addr: *Sockaddr, addrlen: u32): Result<(), Error>
```

#### Dependencies
- builtin
- unsafe

#### Notes
- **Platform Specific:** Different implementations for different OS
- **Unsafe:** All functions are inherently unsafe

### 6.3 process (Process Management - Rust-inspired)
**Status:** ❌ Missing (important for system interaction)
**Description:** Process spawning and management (Rust std::process)

#### Required Types
```vex
struct Command {
    program: str,
    args: Vec<str>,
    env: Option<Vec<(str, str)>>,
    cwd: Option<PathBuf>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
}

enum Stdio {
    Inherit,
    Piped,
    Null,
}

struct Child {
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    // internal process handle
}

struct Output {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

struct ExitStatus {
    code: Option<i32>,
}
```

#### Required Functions
```vex
// Command construction
fn new(program: str): Command
fn arg(cmd: &mut Command, arg: str): &mut Command
fn args(cmd: &mut Command, args: &[str]): &mut Command
fn env<K, V>(cmd: &mut Command, key: K, val: V): &mut Command where K: AsRef<str>, V: AsRef<str>
fn env_clear(cmd: &mut Command)
fn env_remove(cmd: &mut Command, key: str)
fn current_dir(cmd: &mut Command, dir: PathBuf): &mut Command

// I/O redirection
fn stdin<T: Into<Stdio>>(cmd: &mut Command, cfg: T): &mut Command
fn stdout<T: Into<Stdio>>(cmd: &mut Command, cfg: T): &mut Command
fn stderr<T: Into<Stdio>>(cmd: &mut Command, cfg: T): &mut Command

// Execution
fn spawn(cmd: &Command): Result<Child, Error>
fn output(cmd: &Command): Result<Output, Error>
fn status(cmd: &Command): Result<ExitStatus, Error>

// Child process management
fn id(child: &Child): u32
fn kill(child: &mut Child): Result<(), Error>
fn wait(child: &mut Child): Result<ExitStatus, Error>
fn wait_with_output(child: &mut Child): Result<Output, Error>
fn try_wait(child: &mut Child): Result<Option<ExitStatus>, Error>
```

#### Dependencies
- builtin
- io
- path
- env

### 6.4 thread (Thread Management - Rust-inspired)
**Status:** ❌ Missing (low-level threading)
**Description:** Thread creation and management (Rust std::thread)

#### Required Types
```vex
struct JoinHandle<T> {
    // internal thread handle
}

struct Thread {
    // internal thread identifier
}

struct Builder {
    name: Option<str>,
    stack_size: Option<usize>,
}

struct LocalKey<T> {
    // thread-local storage
}

struct AccessError {
    // thread-local access error
}
```

#### Required Functions
```vex
// Thread creation
fn spawn<F, T>(f: F): JoinHandle<T> where F: FnOnce() -> T + Send + 'static, T: Send + 'static
fn spawn_named<F, T>(name: str, f: F): JoinHandle<T> where F: FnOnce() -> T + Send + 'static, T: Send + 'static

// Builder pattern
fn builder(): Builder
fn name(builder: &mut Builder, name: str): &mut Builder
fn stack_size(builder: &mut Builder, size: usize): &mut Builder
fn spawn<F, T>(builder: Builder, f: F): Result<JoinHandle<T>, Error> where F: FnOnce() -> T + Send + 'static, T: Send + 'static

// Thread information
fn current(): Thread
fn park()
fn park_timeout(dur: Duration)
fn unpark(thread: &Thread)

// Thread-local storage
fn local_key<T: 'static>(): &'static LocalKey<T>
fn set<T: 'static>(key: &'static LocalKey<T>, value: T)
fn get<T: 'static>(key: &'static LocalKey<T>): Result<T, AccessError>
fn with<T: 'static, F, R>(key: &'static LocalKey<T>, f: F): R where F: FnOnce(Option<&T>) -> R

// Utility functions
fn yield_now()
fn sleep(dur: Duration)
fn available_parallelism(): Result<usize, Error>

// JoinHandle operations
fn join<T>(handle: JoinHandle<T>): Result<T, Box<dyn Any>>
fn thread(handle: &JoinHandle<T>): &Thread
fn is_finished<T>(handle: &JoinHandle<T>): bool
```

#### Dependencies
- builtin
- time
- panic
- marker

#### Notes
- **Send/Sync Traits:** Thread safety markers gerekli
- **Lifetime Bounds:** 'static lifetime constraints
- **Panic Propagation:** Thread panics handling

### 6.5 path/filepath
**Status:** ❌ Missing (important for path handling)
**Description:** Path manipulation utilities

#### Required Functions
```vex
// Path operations
fn is_abs(path: str): bool
fn abs(path: str): Result<str, Error>
fn rel(basepath: str, targpath: str): Result<str, Error>
fn clean(path: str): str
fn split(path: str): (str, str)
fn split_list(path: str): []str
fn join(elem: ...str): str
fn ext(path: str): str
fn base(path: str): str
fn dir(path: str): str

// Path matching
fn match(pattern: str, name: str): Result<bool, Error>
fn glob(pattern: str): Result<[]str, Error>

// Volume operations
fn volume_name(path: str): str
fn is_path_separator(c: u8): bool
fn from_slash(path: str): str
fn to_slash(path: str): str
```

#### Dependencies
- builtin
- strings
- sort

### 6.6 fs (extend existing)
**Status:** ✅ Exists (extend with missing functionality)
**Description:** Extend existing filesystem package

#### Required Extensions
```vex
// Additional file operations
fn read_file(filename: str): Result<Vec<u8>, Error>
fn write_file(filename: str, data: []u8, perm: u32): Result<(), Error>
fn append_file(filename: str, data: []u8): Result<(), Error>

// Directory operations
fn read_dir(dirname: str): Result<[]DirEntry, Error>
fn walk_dir(root: str, fn(path: str, d: DirEntry, err: Error): WalkDirResult): Error

// Permissions
fn chmod(name: str, mode: FileMode): Result<(), Error>
fn chown(name: str, uid: int, gid: int): Result<(), Error>

// File info
fn same_file(fi1: FileInfo, fi2: FileInfo): bool
fn mode_perm(mode: FileMode): u32
```

#### Dependencies
- builtin
- io
- time

## 🎯 Implementation Priority

1. **os** - Core OS interface
2. **path/filepath** - Path manipulation
3. **fs extensions** - Complete filesystem operations
4. **process** - Process management (Rust-inspired)
5. **thread** - Thread management (Rust-inspired)
6. **syscall** - Low-level system calls

## ⚠️ Language Feature Issues

- **Raw Pointers:** Extensive use of *u8 for C interop
- **Platform Differences:** OS-specific implementations needed
- **Async Operations:** File I/O may need async support

## 📋 Missing Critical Dependencies

- **FFI Bindings:** Extensive C library bindings for system calls
- **Platform Detection:** Compile-time OS detection
- **Error Types:** System-specific error codes

## 🚀 Next Steps

1. Implement basic os package with file operations
2. Add path/filepath utilities
3. Extend fs with directory operations
4. Add syscall package for low-level access