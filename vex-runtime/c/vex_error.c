/**
 * Vex Error Handling
 * Panic and assertion utilities
 */

#include "vex.h"
#include <stdlib.h>

// ============================================================================
// ERROR HANDLING
// ============================================================================

void vex_panic(const char* msg) {
    vex_eprintln("PANIC:");
    vex_eprintln(msg);
    exit(1);
}

void vex_assert(bool cond, const char* msg) {
    if (!cond) {
        vex_panic(msg);
    }
}
