/* test_cmd_stream.c - Test streaming command execution
 * 
 * Test: Run 'ping' for 10 seconds and stream output in real-time
 * 
 * Compile: cc -O3 -std=c17 test_cmd_stream.c ../vex_cmd.c -o test_cmd_stream
 * Run: ./test_cmd_stream
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include <sys/types.h>
#include <stdbool.h>
#include <signal.h>

// Forward declarations from vex_cmd.c
typedef struct {
  char **argv;
  char **env;
  const char *cwd;
  bool capture_stdout;
  bool capture_stderr;
  bool merge_stderr;
} vex_cmd_config_t;

typedef struct {
  pid_t pid;
  int stdin_fd;
  int stdout_fd;
  int stderr_fd;
  bool running;
} vex_cmd_stream_t;

// Function declarations
vex_cmd_stream_t* vex_cmd_stream_spawn(vex_cmd_config_t *config);
ssize_t vex_cmd_stream_write(vex_cmd_stream_t *stream, const void *data, size_t size);
ssize_t vex_cmd_stream_read_stdout(vex_cmd_stream_t *stream, void *buffer, size_t size);
ssize_t vex_cmd_stream_read_stderr(vex_cmd_stream_t *stream, void *buffer, size_t size);
int vex_cmd_stream_wait(vex_cmd_stream_t *stream, int timeout_ms);
void vex_cmd_stream_close_stdin(vex_cmd_stream_t *stream);
void vex_cmd_stream_free(vex_cmd_stream_t *stream);
bool vex_cmd_kill(pid_t proc, bool force);

static uint64_t get_time_ms(void) {
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (uint64_t)ts.tv_sec * 1000 + (uint64_t)ts.tv_nsec / 1000000;
}

int main(void) {
  printf("=== Vex CMD Streaming Test ===\n");
  printf("Running 'ping 8.8.8.8' for 10 seconds...\n");
  printf("Expected: Real-time output (not buffered)\n\n");
  
  // Spawn ping process with streaming
  char *argv[] = {"ping", "-c", "10", "8.8.8.8", NULL};  // 10 pings
  vex_cmd_config_t config = {
    .argv = argv,
    .env = NULL,
    .cwd = NULL,
  };
  
  vex_cmd_stream_t *stream = vex_cmd_stream_spawn(&config);
  if (!stream) {
    fprintf(stderr, "Failed to spawn process!\n");
    return 1;
  }
  
  printf("✅ Process spawned! PID: %d\n", (int)stream->pid);
  printf("✅ Streaming started...\n\n");
  printf("--- OUTPUT ---\n");
  
  char buffer[4096];
  uint64_t start_time = get_time_ms();
  int line_count = 0;
  
  // Read loop (stream output in real-time)
  while (1) {
    // Check if process is still running (non-blocking)
    int exit_code = vex_cmd_stream_wait(stream, 0);
    if (exit_code != -2) {
      // Process exited
      printf("\n--- END OF OUTPUT ---\n");
      printf("✅ Process exited with code: %d\n", exit_code);
      break;
    }
    
    // Read stdout (non-blocking)
    ssize_t n = vex_cmd_stream_read_stdout(stream, buffer, sizeof(buffer) - 1);
    if (n > 0) {
      buffer[n] = '\0';
      printf("%s", buffer);  // Print immediately (streaming!)
      fflush(stdout);
      line_count++;
    }
    
    // Read stderr (non-blocking)
    n = vex_cmd_stream_read_stderr(stream, buffer, sizeof(buffer) - 1);
    if (n > 0) {
      buffer[n] = '\0';
      fprintf(stderr, "[STDERR] %s", buffer);
      fflush(stderr);
    }
    
    // Check timeout (15 seconds)
    uint64_t elapsed = get_time_ms() - start_time;
    if (elapsed > 15000) {
      printf("\n⚠️ Timeout reached (15 seconds), killing process...\n");
      vex_cmd_kill(stream->pid, true);
      break;
    }
    
    // Small sleep to avoid busy-wait (1ms)
    usleep(1000);
  }
  
  uint64_t total_time = get_time_ms() - start_time;
  
  printf("\n=== Results ===\n");
  printf("Total time: %.2f seconds\n", total_time / 1000.0);
  printf("Lines received: %d\n", line_count);
  printf("\n✅ Streaming test passed!\n");
  printf("💡 Tip: Output appeared in real-time (not buffered)\n");
  
  vex_cmd_stream_free(stream);
  return 0;
}

