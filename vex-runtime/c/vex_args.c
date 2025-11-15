// vex_args.c - Command-line arguments support
// Provides argc/argv access to Vex programs
#include "vex.h"
#include <stdlib.h>

// Global storage for command-line arguments
static int global_argc = 0;
static char **global_argv = NULL;

// Initialize global argc/argv (called before main)
void vex_args_init(int argc, char **argv) {
    global_argc = argc;
    global_argv = argv;
}

// Get total argument count (including program name)
int vex_argc(void) {
    return global_argc;
}

// Get argument at index (bounds-checked)
const char *vex_argv(int index) {
    if (index < 0 || index >= global_argc) {
        return NULL;
    }
    return global_argv[index];
}

// Get program name (argv[0])
const char *vex_program_name(void) {
    if (global_argc > 0 && global_argv != NULL) {
        return global_argv[0];
    }
    return "";
}

// Get argument count (excluding program name)
int vex_arg_count(void) {
    return global_argc > 0 ? global_argc - 1 : 0;
}

// ⭐ NEW: Runtime initialization wrapper
// This is called from generated main() to initialize argc/argv
void __vex_runtime_init(int argc, char **argv) {
    vex_args_init(argc, argv);
}
