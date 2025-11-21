#!/bin/bash
# Vex Async Runtime Test Suite Runner

set -e

CC=${CC:-clang}
CFLAGS="-std=c11 -O2 -Wall -Wextra -pthread -DVEX_USE_MIMALLOC -DVEX_ALLOC_TRACKING=1"
INC="-Iinclude -I.. -I../allocators/mimalloc/include"
ALLOC_OBJ="../vex_alloc.o ../allocators/mimalloc/src/static.o"

# Determine poller
UNAME_S=$(uname -s)
if [[ "$UNAME_S" == "Darwin" ]] || [[ "$UNAME_S" == "FreeBSD" ]]; then
    POLLER="kqueue"
    SRC_POLL="src/poller_kqueue.c"
    LIBS=""
elif [[ "$UNAME_S" == "Linux" ]]; then
    KERNEL_VER=$(uname -r | cut -d. -f1-2)
    if (( $(echo "$KERNEL_VER >= 5.11" | bc -l) )); then
        POLLER="io_uring"
        SRC_POLL="src/poller_io_uring.c"
        LIBS="-luring"
    else
        POLLER="epoll"
        SRC_POLL="src/poller_epoll.c"
        LIBS=""
    fi
else
    echo "Unsupported OS: $UNAME_S"
    exit 1
fi

echo "====================================="
echo "Vex Async Runtime Test Suite"
echo "OS: $UNAME_S"
echo "Poller: $POLLER"
echo "====================================="
echo

# Build allocator objects first
if [ ! -f "../vex_alloc.o" ]; then
    echo "Building vex_alloc.o..."
    $CC $CFLAGS $INC -c -o ../vex_alloc.o ../vex_alloc.c
fi
if [ ! -f "../allocators/mimalloc/src/static.o" ]; then
    echo "Building mimalloc..."
    $CC $CFLAGS $INC -c -o ../allocators/mimalloc/src/static.o ../allocators/mimalloc/src/static.c
fi

# Compile runtime sources
SRC_COMMON="src/runtime.c src/worker_context.c src/lockfree_queue.c src/common.c"
SRC_ALL="$SRC_COMMON $SRC_POLL"

# Build object files
OBJ=""
for src in $SRC_ALL; do
    obj="${src%.c}.o"
    echo "Compiling $src..."
    $CC $CFLAGS $INC -c -o "$obj" "$src"
    OBJ="$OBJ $obj"
done

# Test files
TESTS=(
    "test_basic_spawn"
    "test_timer_await"
    "test_local_spawn"
    "test_work_stealing"
    "test_cancel_token"
    "test_lockfree_queue"
    "test_stress"
    "test_real_io_socket"
    "test_edge_cases"
    "test_memory_safety"
    "test_concurrency_bugs"
)

PASSED=0
FAILED=0
TOTAL=${#TESTS[@]}

# Run each test
for test in "${TESTS[@]}"; do
    echo "----------------------------------------"
    echo "Building $test..."
    
    TEST_SRC="tests/${test}.c"
    TEST_BIN="tests/${test}"
    
    if [[ ! -f "$TEST_SRC" ]]; then
        echo "❌ SKIP: $TEST_SRC not found"
        continue
    fi
    
    # Compile test
    if $CC $CFLAGS $INC -o "$TEST_BIN" "$TEST_SRC" $OBJ $ALLOC_OBJ $LIBS; then
        echo "Running $test..."
        echo
        
        # Run test with timeout
        if timeout 10 "$TEST_BIN"; then
            PASSED=$((PASSED + 1))
            echo
        else
            FAILED=$((FAILED + 1))
            echo
            echo "❌ Test failed or timed out"
        fi
    else
        echo "❌ Compilation failed"
        FAILED=$((FAILED + 1))
    fi
    
    echo
done

echo "======================================"
echo "Test Results:"
echo "  ✅ Passed: $PASSED"
echo "  ❌ Failed: $FAILED"
echo "  📊 Total:  $TOTAL"
echo "======================================"

if [[ $FAILED -eq 0 ]]; then
    echo "🎉 All tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed"
    exit 1
fi
