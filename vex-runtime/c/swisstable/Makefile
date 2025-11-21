# SwissTable Makefile - Monolithic Build

CFLAGS := -std=c11 -O3 -march=native -mtune=native -Wall -Wextra -DVEX_USE_MIMALLOC
CFLAGS += -fno-plt -funroll-loops -finline-functions -ffast-math -fomit-frame-pointer
CC := clang
MIMALLOC_DIR := ../allocators/mimalloc
INCLUDES := -I.. -I$(MIMALLOC_DIR)/include

# Platform detection
UNAME_M := $(shell uname -m)

# Sources
SRC := vex_swisstable.c
OBJ := vex_swisstable.o
BENCH_SRC := vex_swisstable_bench.c

# Allocator objects
ALLOC_OBJ := ../vex_alloc.o
MIMALLOC_OBJ := ../mimalloc.o

.PHONY: all bench clean info

all: libvex_swisstable.a

libvex_swisstable.a: $(OBJ)
	ar rcs $@ $(OBJ)
	@echo "📦 Created library: $@"

$(OBJ): $(SRC)
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

# Allocator dependencies
$(MIMALLOC_OBJ):
	$(CC) $(CFLAGS) -I$(MIMALLOC_DIR)/include -DMI_MALLOC_OVERRIDE -c -o $(MIMALLOC_OBJ) $(MIMALLOC_DIR)/src/static.c

$(ALLOC_OBJ): ../vex_alloc.c
	$(CC) $(CFLAGS) -I.. -I$(MIMALLOC_DIR)/include -c -o $(ALLOC_OBJ) ../vex_alloc.c

# Benchmark
bench: bench_main
	@echo ""
	@echo "═══════════════════════════════════════════════════════════"
	@echo "  🏆 SwissTable Benchmark"
	@echo "═══════════════════════════════════════════════════════════"
	@echo ""
	./bench_main

bench_main: $(BENCH_SRC) $(OBJ) $(ALLOC_OBJ) $(MIMALLOC_OBJ)
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ $(BENCH_SRC) $(OBJ) $(ALLOC_OBJ) $(MIMALLOC_OBJ)

clean:
	rm -f *.o *.a bench_main
	@echo "✅ Clean complete"

info:
	@echo "Platform: $(UNAME_M)"
	@echo "Compiler: $(CC)"
