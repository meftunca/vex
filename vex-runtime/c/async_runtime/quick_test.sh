#!/bin/bash
# Quick test runner (skips slow tests)

set -e

CC=${CC:-clang}
CFLAGS="-std=c11 -O2 -Wall -Wextra -pthread -DVEX_USE_MIMALLOC -DVEX_ALLOC_TRACKING=1"
INC="-Iinclude -I.. -I../allocators/mimalloc/include"
ALLOC_OBJ="../vex_alloc.o ../allocators/mimalloc/src/static.o"

UNAME_S=$(uname -s)
if [[ "$UNAME_S" == "Darwin" ]] || [[ "$UNAME_S" == "FreeBSD" ]]; then
    POLLER="kqueue"
    SRC_POLL="src/poller_kqueue.c"
    LIBS=""
elif [[ "$UNAME_S" == "Linux" ]]; then
    POLLER="epoll"
    SRC_POLL="src/poller_epoll.c"
    LIBS=""
fi

echo "Quick Test Suite (OS: $UNAME_S, Poller: $POLLER)"
echo "================================================"

# Build allocator objects first
if [ ! -f "../vex_alloc.o" ]; then
    $CC $CFLAGS $INC -c -o ../vex_alloc.o ../vex_alloc.c 2>/dev/null
fi
if [ ! -f "../allocators/mimalloc/src/static.o" ]; then
    $CC $CFLAGS $INC -c -o ../allocators/mimalloc/src/static.o ../allocators/mimalloc/src/static.c 2>/dev/null
fi

SRC_COMMON="src/runtime.c src/worker_context.c src/lockfree_queue.c src/common.c src/timer_heap.c"
SRC_ALL="$SRC_COMMON $SRC_POLL"

OBJ=""
for src in $SRC_ALL; do
    obj="${src%.c}.o"
    $CC $CFLAGS $INC -c -o "$obj" "$src" 2>/dev/null
    OBJ="$OBJ $obj"
done

# Core tests only
TESTS=(
    "test_basic_spawn"
    "test_lockfree_queue"
    "test_edge_cases"
    "test_memory_safety"
    "test_concurrency_bugs"
)

PASSED=0
FAILED=0

for test in "${TESTS[@]}"; do
    TEST_SRC="tests/${test}.c"
    TEST_BIN="tests/${test}"
    
    printf "%-30s" "$test..."
    
    if $CC $CFLAGS $INC -o "$TEST_BIN" "$TEST_SRC" $OBJ $ALLOC_OBJ $LIBS 2>/dev/null; then
        if timeout 5 "$TEST_BIN" >/dev/null 2>&1; then
            echo "✅"
            PASSED=$((PASSED + 1))
        else
            echo "❌"
            FAILED=$((FAILED + 1))
        fi
    else
        echo "❌ (compile)"
        FAILED=$((FAILED + 1))
    fi
done

echo "================================================"
echo "Results: ✅ $PASSED passed, ❌ $FAILED failed"

exit $FAILED
