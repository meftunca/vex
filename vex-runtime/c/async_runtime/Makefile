CC = clang
CFLAGS = -std=c11 -O2 -Wall -Wextra -pthread
# Enable mimalloc and allocation tracking for allocator consistency with the rest of the runtime
CFLAGS += -DVEX_USE_MIMALLOC -DVEX_ALLOC_TRACKING=1
UNAME_S := $(shell uname -s)
KERNEL_R := $(shell uname -r)
KERNEL_MAJ := $(word 1, $(subst ., ,$(KERNEL_R)))
KERNEL_MIN := $(word 2, $(subst ., ,$(KERNEL_R)))

# vex_net integration
VEXNET_DIR = ../vex_net
VEXNET_LIB = $(VEXNET_DIR)/libvexnet.a
VEXNET_INC = -I$(VEXNET_DIR)/include

# Use vex_net if USE_VEXNET=1
ifdef USE_VEXNET
    POLLER = vexnet
else
    # default poller per-OS
    ifeq ($(UNAME_S),Linux)
        # decide io_uring vs epoll
        # if kernel >= 5.11 -> io_uring, else epoll
        CMP_GE_5_11 := $(shell [ $(KERNEL_MAJ) -gt 5 -o \( $(KERNEL_MAJ) -eq 5 -a $(KERNEL_MIN) -ge 11 \) ] && echo yes || echo no)
        ifeq ($(CMP_GE_5_11),yes)
            POLLER ?= io_uring
        else
            POLLER ?= epoll
        endif
    endif

    ifeq ($(UNAME_S),Darwin)
        POLLER ?= kqueue
    endif

    ifeq ($(UNAME_S),FreeBSD)
        POLLER ?= kqueue
    endif

    ifeq ($(OS),Windows_NT)
        POLLER ?= iocp
    endif
endif

SRC_COMMON = src/runtime.c src/worker_context.c src/lockfree_queue.c src/common.c src/timer_heap.c src/task_pool.c

# Ensure we link the core allocator implementation so async runtime tests can use vex_malloc/vex_free
ALLOC_SRC = ../vex_alloc.c
ALLOC_OBJ = ../vex_alloc.o

ifeq ($(POLLER),vexnet)
    SRC_POLL = src/poller_vexnet.c
    CFLAGS += $(VEXNET_INC)
    LIBS = $(VEXNET_LIB)
endif
ifeq ($(POLLER),io_uring)
    SRC_POLL = src/poller_io_uring.c
    LIBS = -luring
endif
ifeq ($(POLLER),epoll)
    SRC_POLL = src/poller_epoll.c
endif
ifeq ($(POLLER),kqueue)
    SRC_POLL = src/poller_kqueue.c
endif
ifeq ($(POLLER),iocp)
    SRC_POLL = src/poller_iocp.c
endif

SRC = $(SRC_COMMON) $(SRC_POLL)
INC = -Iinclude
INC += -I..
INC += -I../allocators/mimalloc/include
OBJ = $(SRC:.c=.o)
TARGET = async_runtime_demo
MIMALLOC_SRC = ../allocators/mimalloc/src/static.c
MIMALLOC_OBJ = ../allocators/mimalloc/src/static.o

all: $(TARGET)

# Build vex_net if using vexnet poller
ifeq ($(POLLER),vexnet)
$(VEXNET_LIB):
	$(MAKE) -C $(VEXNET_DIR)
else
$(VEXNET_LIB):
	@echo "vex_net not needed for $(POLLER)"
endif

$(TARGET): $(VEXNET_LIB) $(OBJ) tests/example_async_demo.o $(ALLOC_OBJ) $(MIMALLOC_OBJ)
	$(CC) $(CFLAGS) -o $@ $(OBJ) tests/example_async_demo.o $(ALLOC_OBJ) $(MIMALLOC_OBJ) $(LIBS)

tests/example_async_demo.o: tests/example_async_demo.c include/runtime.h include/poller.h
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

%.o: %.c
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

# Test vex_net integration
test_vexnet_integration: $(VEXNET_LIB) $(OBJ) test_vexnet_integration.o $(ALLOC_OBJ) $(MIMALLOC_OBJ)
	$(CC) $(CFLAGS) -o $@ $(OBJ) test_vexnet_integration.o $(ALLOC_OBJ) $(MIMALLOC_OBJ) $(LIBS)

test_vexnet_integration.o: test_vexnet_integration.c
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

# Simple test without timers
test_simple_vexnet: $(VEXNET_LIB) $(OBJ) test_simple_vexnet.o $(ALLOC_OBJ) $(MIMALLOC_OBJ)
	$(CC) $(CFLAGS) -o $@ $(OBJ) test_simple_vexnet.o $(ALLOC_OBJ) $(MIMALLOC_OBJ) $(LIBS)

test_simple_vexnet.o: test_simple_vexnet.c
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

# Demo without timer dependency
demo_notimer: $(VEXNET_LIB) $(OBJ) demo_notimer.o $(ALLOC_OBJ) $(MIMALLOC_OBJ)
	$(CC) $(CFLAGS) -o $@ $(OBJ) demo_notimer.o $(ALLOC_OBJ) $(MIMALLOC_OBJ) $(LIBS)

demo_notimer.o: demo_notimer.c
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

# Demo WITH timer (proper state machine)
demo_with_timer: $(VEXNET_LIB) $(OBJ) demo_with_timer.o $(ALLOC_OBJ) $(MIMALLOC_OBJ)
	$(CC) $(CFLAGS) -o $@ $(OBJ) demo_with_timer.o $(ALLOC_OBJ) $(MIMALLOC_OBJ) $(LIBS)

demo_with_timer.o: demo_with_timer.c
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

clean:
	@echo "Cleaning build artifacts..."
	@rm -f $(OBJ) tests/example_async_demo.o $(TARGET) test_vexnet_integration test_vexnet_integration.o \
		test_simple_vexnet test_simple_vexnet.o demo_notimer demo_notimer.o demo_with_timer demo_with_timer.o
	@rm -f $(ALLOC_OBJ)
	@rm -f $(MIMALLOC_OBJ)

clean-all: clean
	$(MAKE) -C $(VEXNET_DIR) clean

print-config:
	@echo OS=$(UNAME_S)
	@echo KERNEL=$(KERNEL_R)
	@echo POLLER=$(POLLER)
	@echo USE_VEXNET=$(USE_VEXNET)
	@echo VEXNET_LIB=$(VEXNET_LIB)

# PRO extensions: timers/cancel/stats are built-in to runtime.c stubs

../vex_alloc.o: ../vex_alloc.c
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

$(MIMALLOC_OBJ): $(MIMALLOC_SRC)
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

# Synthetic benchmark (pure scheduling test)
benchmark_synthetic: $(VEXNET_LIB) $(OBJ) benchmark_synthetic.o $(ALLOC_OBJ) $(MIMALLOC_OBJ)
	$(CC) $(CFLAGS) -o $@ $(OBJ) benchmark_synthetic.o $(ALLOC_OBJ) $(MIMALLOC_OBJ) $(LIBS)

benchmark_synthetic.o: benchmark_synthetic.c
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

# Connection-pooled network benchmark
benchmark_pooled: $(VEXNET_LIB) $(OBJ) benchmark_pooled.o $(ALLOC_OBJ) $(MIMALLOC_OBJ)
	$(CC) $(CFLAGS) -o $@ $(OBJ) benchmark_pooled.o $(ALLOC_OBJ) $(MIMALLOC_OBJ) $(LIBS)

benchmark_pooled.o: benchmark_pooled.c
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<
