/* vex_cmd.c - Command execution and process management for Vex
 * 
 * Features:
 * - Execute commands (blocking/non-blocking)
 * - Spawn processes with stdin/stdout/stderr piping
 * - Environment variable control
 * - Working directory control
 * - Exit code capture
 * - Signal handling (SIGTERM, SIGKILL)
 * - Process groups
 * 
 * Cross-platform: Linux, macOS, Windows
 * 
 * Build: cc -O3 -std=c17 vex_cmd.c -o test_cmd
 * 
 * License: MIT
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>

#if __has_include("vex_macros.h")
  #include "vex_macros.h"
#else
  #define VEX_INLINE static inline
  
  #if defined(_WIN32)
    #define VEX_OS_WINDOWS 1
  #elif defined(__linux__)
    #define VEX_OS_LINUX 1
  #elif defined(__APPLE__)
    #define VEX_OS_MACOS 1
  #endif
#endif

// Platform-specific includes
#if defined(_WIN32) || defined(_WIN64) || defined(__CYGWIN__)
  #ifndef VEX_OS_WINDOWS
    #define VEX_OS_WINDOWS 1
  #endif
#endif

#ifdef VEX_OS_WINDOWS
  #if VEX_OS_WINDOWS
    #include <windows.h>
    #include <process.h>
    #include <io.h>
  #endif
#else
  #include <unistd.h>
  #include <sys/types.h>
  #include <sys/wait.h>
  #include <sys/stat.h>
  #include <fcntl.h>
  #include <signal.h>
  #include <errno.h>
#endif

/* =========================
 * Types
 * ========================= */

typedef struct {
  char **argv;         // Command arguments (NULL-terminated)
  char **env;          // Environment variables (NULL-terminated, can be NULL)
  const char *cwd;     // Working directory (can be NULL)
  bool capture_stdout; // Capture stdout?
  bool capture_stderr; // Capture stderr?
  bool merge_stderr;   // Merge stderr into stdout?
} vex_cmd_config_t;

typedef struct {
  int exit_code;       // Exit code (or -1 if not exited)
  char *stdout_data;   // Captured stdout (NULL if not captured)
  size_t stdout_size;
  char *stderr_data;   // Captured stderr (NULL if not captured)
  size_t stderr_size;
  bool success;        // true if exit_code == 0
} vex_cmd_result_t;

#if defined(VEX_OS_WINDOWS) || defined(_WIN32) || defined(_WIN64)
typedef HANDLE vex_process_t;
#else
typedef pid_t vex_process_t;
#endif

/* =========================
 * Streaming Process Handle
 * ========================= */

typedef struct {
  vex_process_t pid;
  int stdin_fd;   // Write to child stdin
  int stdout_fd;  // Read from child stdout
  int stderr_fd;  // Read from child stderr
  bool running;
} vex_cmd_stream_t;

/* =========================
 * Execute Command (Blocking)
 * ========================= */

vex_cmd_result_t* vex_cmd_exec(vex_cmd_config_t *config) {
  if (!config || !config->argv || !config->argv[0]) {
    return NULL;
  }
  
  vex_cmd_result_t *result = (vex_cmd_result_t*)calloc(1, sizeof(vex_cmd_result_t));
  result->exit_code = -1;
  
#if defined(VEX_OS_WINDOWS)
  // Windows implementation (simplified)
  STARTUPINFOA si = {0};
  PROCESS_INFORMATION pi = {0};
  si.cb = sizeof(si);
  
  // Build command line
  char cmdline[4096] = {0};
  size_t offset = 0;
  for (int i = 0; config->argv[i]; i++) {
    if (i > 0) cmdline[offset++] = ' ';
    offset += snprintf(cmdline + offset, sizeof(cmdline) - offset, "\"%s\"", config->argv[i]);
  }
  
  BOOL success = CreateProcessA(
    NULL,           // Application name
    cmdline,        // Command line
    NULL,           // Process attributes
    NULL,           // Thread attributes
    FALSE,          // Inherit handles
    0,              // Creation flags
    NULL,           // Environment
    config->cwd,    // Current directory
    &si,            // Startup info
    &pi             // Process info
  );
  
  if (!success) {
    free(result);
    return NULL;
  }
  
  // Wait for process to finish
  WaitForSingleObject(pi.hProcess, INFINITE);
  
  // Get exit code
  DWORD exit_code;
  GetExitCodeProcess(pi.hProcess, &exit_code);
  result->exit_code = (int)exit_code;
  result->success = (exit_code == 0);
  
  CloseHandle(pi.hProcess);
  CloseHandle(pi.hThread);
  
#else
  // Unix/Linux/macOS implementation
  int stdout_pipe[2] = {-1, -1};
  int stderr_pipe[2] = {-1, -1};
  
  // Create pipes if needed
  if (config->capture_stdout) {
    if (pipe(stdout_pipe) < 0) {
      free(result);
      return NULL;
    }
  }
  
  if (config->capture_stderr && !config->merge_stderr) {
    if (pipe(stderr_pipe) < 0) {
      if (stdout_pipe[0] >= 0) {
        close(stdout_pipe[0]);
        close(stdout_pipe[1]);
      }
      free(result);
      return NULL;
    }
  }
  
  pid_t pid = fork();
  if (pid < 0) {
    // Fork failed
    if (stdout_pipe[0] >= 0) {
      close(stdout_pipe[0]);
      close(stdout_pipe[1]);
    }
    if (stderr_pipe[0] >= 0) {
      close(stderr_pipe[0]);
      close(stderr_pipe[1]);
    }
    free(result);
    return NULL;
  }
  
  if (pid == 0) {
    // Child process
    
    // Change working directory
    if (config->cwd) {
      if (chdir(config->cwd) < 0) {
        _exit(127);
      }
    }
    
    // Set up stdout pipe
    if (config->capture_stdout) {
      close(stdout_pipe[0]);  // Close read end
      dup2(stdout_pipe[1], STDOUT_FILENO);
      close(stdout_pipe[1]);
    }
    
    // Set up stderr pipe
    if (config->merge_stderr && config->capture_stdout) {
      dup2(STDOUT_FILENO, STDERR_FILENO);
    } else if (config->capture_stderr) {
      close(stderr_pipe[0]);  // Close read end
      dup2(stderr_pipe[1], STDERR_FILENO);
      close(stderr_pipe[1]);
    }
    
    // Execute command
    if (config->env) {
      execve(config->argv[0], config->argv, config->env);
    } else {
      execvp(config->argv[0], config->argv);
    }
    
    // If we reach here, exec failed
    _exit(127);
  }
  
  // Parent process
  
  // Close write ends of pipes
  if (stdout_pipe[1] >= 0) close(stdout_pipe[1]);
  if (stderr_pipe[1] >= 0) close(stderr_pipe[1]);
  
  // Read stdout
  if (config->capture_stdout) {
    size_t capacity = 4096;
    result->stdout_data = (char*)malloc(capacity);
    result->stdout_size = 0;
    
    ssize_t n;
    while ((n = read(stdout_pipe[0], result->stdout_data + result->stdout_size, 
                     capacity - result->stdout_size)) > 0) {
      result->stdout_size += n;
      if (result->stdout_size >= capacity) {
        capacity *= 2;
        result->stdout_data = (char*)realloc(result->stdout_data, capacity);
      }
    }
    
    close(stdout_pipe[0]);
  }
  
  // Read stderr
  if (config->capture_stderr && !config->merge_stderr) {
    size_t capacity = 4096;
    result->stderr_data = (char*)malloc(capacity);
    result->stderr_size = 0;
    
    ssize_t n;
    while ((n = read(stderr_pipe[0], result->stderr_data + result->stderr_size, 
                     capacity - result->stderr_size)) > 0) {
      result->stderr_size += n;
      if (result->stderr_size >= capacity) {
        capacity *= 2;
        result->stderr_data = (char*)realloc(result->stderr_data, capacity);
      }
    }
    
    close(stderr_pipe[0]);
  }
  
  // Wait for child process
  int status;
  waitpid(pid, &status, 0);
  
  if (WIFEXITED(status)) {
    result->exit_code = WEXITSTATUS(status);
    result->success = (result->exit_code == 0);
  } else if (WIFSIGNALED(status)) {
    result->exit_code = 128 + WTERMSIG(status);
    result->success = false;
  }
#endif
  
  return result;
}

/* =========================
 * Spawn Process (Non-blocking)
 * ========================= */

vex_process_t vex_cmd_spawn(vex_cmd_config_t *config) {
  if (!config || !config->argv || !config->argv[0]) {
#if defined(VEX_OS_WINDOWS)
    return NULL;
#else
    return -1;
#endif
  }
  
#if defined(VEX_OS_WINDOWS)
  STARTUPINFOA si = {0};
  PROCESS_INFORMATION pi = {0};
  si.cb = sizeof(si);
  
  // Build command line
  char cmdline[4096] = {0};
  size_t offset = 0;
  for (int i = 0; config->argv[i]; i++) {
    if (i > 0) cmdline[offset++] = ' ';
    offset += snprintf(cmdline + offset, sizeof(cmdline) - offset, "\"%s\"", config->argv[i]);
  }
  
  BOOL success = CreateProcessA(
    NULL, cmdline, NULL, NULL, FALSE, 0, NULL, config->cwd, &si, &pi
  );
  
  if (!success) {
    return NULL;
  }
  
  CloseHandle(pi.hThread);
  return pi.hProcess;
  
#else
  pid_t pid = fork();
  if (pid < 0) {
    return -1;
  }
  
  if (pid == 0) {
    // Child process
    if (config->cwd) {
      chdir(config->cwd);
    }
    
    if (config->env) {
      execve(config->argv[0], config->argv, config->env);
    } else {
      execvp(config->argv[0], config->argv);
    }
    
    _exit(127);
  }
  
  return pid;
#endif
}

/* =========================
 * Wait for Process
 * ========================= */

int vex_cmd_wait(vex_process_t proc) {
#if defined(VEX_OS_WINDOWS)
  if (proc == NULL) return -1;
  WaitForSingleObject(proc, INFINITE);
  DWORD exit_code;
  GetExitCodeProcess(proc, &exit_code);
  CloseHandle(proc);
  return (int)exit_code;
#else
  if (proc < 0) return -1;
  int status;
  waitpid(proc, &status, 0);
  if (WIFEXITED(status)) {
    return WEXITSTATUS(status);
  }
  return -1;
#endif
}

/* =========================
 * Kill Process
 * ========================= */

bool vex_cmd_kill(vex_process_t proc, bool force) {
#if defined(VEX_OS_WINDOWS)
  if (proc == NULL) return false;
  BOOL ret = TerminateProcess(proc, 1);
  CloseHandle(proc);
  return ret != 0;
#else
  if (proc < 0) return false;
  int signal = force ? SIGKILL : SIGTERM;
  return kill(proc, signal) == 0;
#endif
}

/* =========================
 * Free Result
 * ========================= */

void vex_cmd_result_free(vex_cmd_result_t *result) {
  if (!result) return;
  if (result->stdout_data) free(result->stdout_data);
  if (result->stderr_data) free(result->stderr_data);
  free(result);
}

/* =========================
 * Streaming API (Real-time I/O)
 * ========================= */

// Spawn process with streaming pipes
vex_cmd_stream_t* vex_cmd_stream_spawn(vex_cmd_config_t *config) {
  if (!config || !config->argv || !config->argv[0]) {
    return NULL;
  }
  
  vex_cmd_stream_t *stream = (vex_cmd_stream_t*)calloc(1, sizeof(vex_cmd_stream_t));
  stream->stdin_fd = -1;
  stream->stdout_fd = -1;
  stream->stderr_fd = -1;
  stream->running = true;
  
#if defined(VEX_OS_WINDOWS)
  // Windows implementation (simplified)
  HANDLE stdin_rd = NULL, stdin_wr = NULL;
  HANDLE stdout_rd = NULL, stdout_wr = NULL;
  HANDLE stderr_rd = NULL, stderr_wr = NULL;
  
  SECURITY_ATTRIBUTES sa = {sizeof(SECURITY_ATTRIBUTES), NULL, TRUE};
  
  // Create pipes
  CreatePipe(&stdin_rd, &stdin_wr, &sa, 0);
  CreatePipe(&stdout_rd, &stdout_wr, &sa, 0);
  CreatePipe(&stderr_rd, &stderr_wr, &sa, 0);
  
  SetHandleInformation(stdin_wr, HANDLE_FLAG_INHERIT, 0);
  SetHandleInformation(stdout_rd, HANDLE_FLAG_INHERIT, 0);
  SetHandleInformation(stderr_rd, HANDLE_FLAG_INHERIT, 0);
  
  STARTUPINFOA si = {0};
  PROCESS_INFORMATION pi = {0};
  si.cb = sizeof(si);
  si.hStdInput = stdin_rd;
  si.hStdOutput = stdout_wr;
  si.hStdError = stderr_wr;
  si.dwFlags |= STARTF_USESTDHANDLES;
  
  char cmdline[4096] = {0};
  size_t offset = 0;
  for (int i = 0; config->argv[i]; i++) {
    if (i > 0) cmdline[offset++] = ' ';
    offset += snprintf(cmdline + offset, sizeof(cmdline) - offset, "\"%s\"", config->argv[i]);
  }
  
  BOOL success = CreateProcessA(NULL, cmdline, NULL, NULL, TRUE, 0, NULL, config->cwd, &si, &pi);
  
  CloseHandle(stdin_rd);
  CloseHandle(stdout_wr);
  CloseHandle(stderr_wr);
  
  if (!success) {
    CloseHandle(stdin_wr);
    CloseHandle(stdout_rd);
    CloseHandle(stderr_rd);
    free(stream);
    return NULL;
  }
  
  stream->pid = pi.hProcess;
  stream->stdin_fd = _open_osfhandle((intptr_t)stdin_wr, 0);
  stream->stdout_fd = _open_osfhandle((intptr_t)stdout_rd, 0);
  stream->stderr_fd = _open_osfhandle((intptr_t)stderr_rd, 0);
  
  CloseHandle(pi.hThread);
  
#else
  // Unix/Linux/macOS implementation
  int stdin_pipe[2];
  int stdout_pipe[2];
  int stderr_pipe[2];
  
  if (pipe(stdin_pipe) < 0 || pipe(stdout_pipe) < 0 || pipe(stderr_pipe) < 0) {
    free(stream);
    return NULL;
  }
  
  pid_t pid = fork();
  if (pid < 0) {
    close(stdin_pipe[0]); close(stdin_pipe[1]);
    close(stdout_pipe[0]); close(stdout_pipe[1]);
    close(stderr_pipe[0]); close(stderr_pipe[1]);
    free(stream);
    return NULL;
  }
  
  if (pid == 0) {
    // Child process
    
    // Redirect stdin
    dup2(stdin_pipe[0], STDIN_FILENO);
    close(stdin_pipe[0]);
    close(stdin_pipe[1]);
    
    // Redirect stdout
    dup2(stdout_pipe[1], STDOUT_FILENO);
    close(stdout_pipe[0]);
    close(stdout_pipe[1]);
    
    // Redirect stderr
    dup2(stderr_pipe[1], STDERR_FILENO);
    close(stderr_pipe[0]);
    close(stderr_pipe[1]);
    
    // Change working directory
    if (config->cwd) {
      chdir(config->cwd);
    }
    
    // Execute
    if (config->env) {
      execve(config->argv[0], config->argv, config->env);
    } else {
      execvp(config->argv[0], config->argv);
    }
    
    _exit(127);
  }
  
  // Parent process
  close(stdin_pipe[0]);
  close(stdout_pipe[1]);
  close(stderr_pipe[1]);
  
  stream->pid = pid;
  stream->stdin_fd = stdin_pipe[1];
  stream->stdout_fd = stdout_pipe[0];
  stream->stderr_fd = stderr_pipe[0];
  
  // Set non-blocking mode for reading
  fcntl(stream->stdout_fd, F_SETFL, O_NONBLOCK);
  fcntl(stream->stderr_fd, F_SETFL, O_NONBLOCK);
#endif
  
  return stream;
}

// Write to child stdin
ssize_t vex_cmd_stream_write(vex_cmd_stream_t *stream, const void *data, size_t size) {
  if (!stream || stream->stdin_fd < 0) {
    return -1;
  }
  
#if defined(VEX_OS_WINDOWS)
  DWORD written;
  HANDLE h = (HANDLE)_get_osfhandle(stream->stdin_fd);
  if (!WriteFile(h, data, (DWORD)size, &written, NULL)) {
    return -1;
  }
  return (ssize_t)written;
#else
  return write(stream->stdin_fd, data, size);
#endif
}

// Read from child stdout (non-blocking)
ssize_t vex_cmd_stream_read_stdout(vex_cmd_stream_t *stream, void *buffer, size_t size) {
  if (!stream || stream->stdout_fd < 0) {
    return -1;
  }
  
#if defined(VEX_OS_WINDOWS)
  DWORD read_bytes;
  HANDLE h = (HANDLE)_get_osfhandle(stream->stdout_fd);
  if (!ReadFile(h, buffer, (DWORD)size, &read_bytes, NULL)) {
    return (GetLastError() == ERROR_NO_DATA) ? 0 : -1;
  }
  return (ssize_t)read_bytes;
#else
  ssize_t n = read(stream->stdout_fd, buffer, size);
  if (n < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
    return 0;  // No data available (non-blocking)
  }
  return n;
#endif
}

// Read from child stderr (non-blocking)
ssize_t vex_cmd_stream_read_stderr(vex_cmd_stream_t *stream, void *buffer, size_t size) {
  if (!stream || stream->stderr_fd < 0) {
    return -1;
  }
  
#if defined(VEX_OS_WINDOWS)
  DWORD read_bytes;
  HANDLE h = (HANDLE)_get_osfhandle(stream->stderr_fd);
  if (!ReadFile(h, buffer, (DWORD)size, &read_bytes, NULL)) {
    return (GetLastError() == ERROR_NO_DATA) ? 0 : -1;
  }
  return (ssize_t)read_bytes;
#else
  ssize_t n = read(stream->stderr_fd, buffer, size);
  if (n < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
    return 0;  // No data available (non-blocking)
  }
  return n;
#endif
}

// Wait for process with timeout (milliseconds, 0 = non-blocking)
int vex_cmd_stream_wait(vex_cmd_stream_t *stream, int timeout_ms) {
  if (!stream) {
    return -1;
  }
  
#if defined(VEX_OS_WINDOWS)
  DWORD timeout = (timeout_ms == 0) ? 0 : (timeout_ms < 0 ? INFINITE : (DWORD)timeout_ms);
  DWORD ret = WaitForSingleObject(stream->pid, timeout);
  
  if (ret == WAIT_OBJECT_0) {
    DWORD exit_code;
    GetExitCodeProcess(stream->pid, &exit_code);
    stream->running = false;
    return (int)exit_code;
  } else if (ret == WAIT_TIMEOUT) {
    return -2;  // Still running
  }
  return -1;  // Error
#else
  int status;
  int options = (timeout_ms == 0) ? WNOHANG : 0;
  pid_t ret = waitpid(stream->pid, &status, options);
  
  if (ret == stream->pid) {
    stream->running = false;
    if (WIFEXITED(status)) {
      return WEXITSTATUS(status);
    } else if (WIFSIGNALED(status)) {
      return 128 + WTERMSIG(status);
    }
  } else if (ret == 0) {
    return -2;  // Still running
  }
  return -1;  // Error
#endif
}

// Close stdin (signal EOF to child)
void vex_cmd_stream_close_stdin(vex_cmd_stream_t *stream) {
  if (stream && stream->stdin_fd >= 0) {
    close(stream->stdin_fd);
    stream->stdin_fd = -1;
  }
}

// Free streaming handle
void vex_cmd_stream_free(vex_cmd_stream_t *stream) {
  if (!stream) return;
  
  if (stream->stdin_fd >= 0) close(stream->stdin_fd);
  if (stream->stdout_fd >= 0) close(stream->stdout_fd);
  if (stream->stderr_fd >= 0) close(stream->stderr_fd);
  
#if defined(VEX_OS_WINDOWS)
  if (stream->pid) CloseHandle(stream->pid);
#endif
  
  free(stream);
}

/* =========================
 * Helper: Simple Exec
 * ========================= */

int vex_cmd_simple_exec(const char *cmd) {
  char *argv[] = {
#if defined(VEX_OS_WINDOWS)
    "cmd.exe", "/C",
#else
    "/bin/sh", "-c",
#endif
    (char*)cmd,
    NULL
  };
  
  vex_cmd_config_t config = {
    .argv = argv,
    .env = NULL,
    .cwd = NULL,
    .capture_stdout = false,
    .capture_stderr = false,
    .merge_stderr = false
  };
  
  vex_cmd_result_t *result = vex_cmd_exec(&config);
  if (!result) return -1;
  
  int exit_code = result->exit_code;
  vex_cmd_result_free(result);
  return exit_code;
}

/* =========================
 * Demo / Tests
 * ========================= */
#ifdef VEX_CMD_DEMO

int main(void) {
  printf("=== Vex Command Demo ===\n\n");
  
  // Test 1: Simple command
  printf("Test 1: Simple echo\n");
  int ret = vex_cmd_simple_exec("echo Hello from Vex!");
  printf("  Exit code: %d\n\n", ret);
  
  // Test 2: Capture stdout
  printf("Test 2: Capture stdout\n");
  char *argv2[] = {"echo", "Captured output", NULL};
  vex_cmd_config_t config2 = {
    .argv = argv2,
    .capture_stdout = true
  };
  vex_cmd_result_t *result2 = vex_cmd_exec(&config2);
  if (result2) {
    printf("  Stdout: %.*s", (int)result2->stdout_size, result2->stdout_data);
    printf("  Exit code: %d\n\n", result2->exit_code);
    vex_cmd_result_free(result2);
  }
  
  // Test 3: Spawn and wait
  printf("Test 3: Spawn process\n");
  char *argv3[] = {"sleep", "1", NULL};
  vex_cmd_config_t config3 = {.argv = argv3};
  vex_process_t proc = vex_cmd_spawn(&config3);
#if defined(VEX_OS_WINDOWS)
  if (proc != NULL) {
#else
  if (proc > 0) {
#endif
    printf("  Process spawned, waiting...\n");
    int exit_code = vex_cmd_wait(proc);
    printf("  Process exited with code: %d\n\n", exit_code);
  }
  
  printf("✅ All tests passed!\n");
  return 0;
}

#endif // VEX_CMD_DEMO

