# Vex Runtime C Library - Main Makefile
# Provides unified build system for all components

CC := clang
CFLAGS := -std=c11 -O3 -Wall -Wextra -march=native
CFLAGS_TEST := -std=c11 -O2 -Wall -Wextra
INCLUDES := -I. -Iasync_runtime/include -Ivex_time/include -Ivex_net/include
LIBS := -pthread -lm

# Directories
SRC_DIR := .
TEST_DIR := tests
UNIT_TEST_DIR := $(TEST_DIR)/unit
INTEGRATION_TEST_DIR := $(TEST_DIR)/integration
BENCH_DIR := $(TEST_DIR)/benchmarks
BUILD_DIR := build

# Source files (runtime core)
CORE_SRCS := vex_alloc.c vex_array.c vex_box.c vex_cpu.c vex_error.c \
	vex_file.c vex_io.c vex_memory.c vex_mmap.c vex_option.c \
	vex_path.c vex_result.c vex_string.c vex_string_type.c \
	vex_strconv.c vex_url.c vex_vec.c vex_time.c vex_channel.c \
	vex_range.c vex_set.c vex_slice.c vex_tuple.c

# Component libraries
VEXTIME_LIB := vex_time/libvextime.a
VEXNET_LIB := vex_net/libvexnet.a
SWISSTABLE_LIB := swisstable/libvex_swisstable.a

# All test executables
UNIT_TESTS := $(patsubst $(UNIT_TEST_DIR)/%.c,$(UNIT_TEST_DIR)/%,$(wildcard $(UNIT_TEST_DIR)/*.c))
INTEGRATION_TESTS := $(patsubst $(INTEGRATION_TEST_DIR)/%.c,$(INTEGRATION_TEST_DIR)/%,$(wildcard $(INTEGRATION_TEST_DIR)/*.c))

.PHONY: all clean test test-unit test-integration help libs

# Default target
all: libs
	@echo ""
	@echo "═══════════════════════════════════════════════════════════"
	@echo "  ✅ Vex Runtime Build Complete!"
	@echo "═══════════════════════════════════════════════════════════"
	@echo ""
	@echo "Available targets:"
	@echo "  make test           - Run all tests"
	@echo "  make test-unit      - Run unit tests only"
	@echo "  make test-integration - Run integration tests only"
	@echo "  make libs           - Build all component libraries"
	@echo "  make clean          - Clean all build artifacts"
	@echo ""

# Help target
help:
	@echo "Vex Runtime C Library - Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all               - Build all libraries (default)"
	@echo "  libs              - Build component libraries"
	@echo "  test              - Run all tests"
	@echo "  test-unit         - Run unit tests"
	@echo "  test-integration  - Run integration tests"
	@echo "  clean             - Clean build artifacts"
	@echo "  help              - Show this help"
	@echo ""
	@echo "Component-specific targets:"
	@echo "  make -C vex_time test       - Test vex_time"
	@echo "  make -C vex_net test        - Test vex_net"
	@echo "  make -C swisstable test     - Test SwissTable"
	@echo "  make -C async_runtime test  - Test async runtime"

# Build component libraries
libs: $(VEXTIME_LIB) $(SWISSTABLE_LIB)
	@echo "✅ All component libraries built"

$(VEXTIME_LIB):
	@echo "Building vex_time..."
	@cd vex_time && $(MAKE) CC=$(CC) CFLAGS="$(CFLAGS)"

$(VEXNET_LIB):
	@echo "Building vex_net..."
	@cd vex_net && $(MAKE) CC=$(CC) CFLAGS="$(CFLAGS)"

$(SWISSTABLE_LIB):
	@echo "Building SwissTable..."
	@cd swisstable && ./build_swisstable.sh

# Build unit tests
$(UNIT_TEST_DIR)/%: $(UNIT_TEST_DIR)/%.c vex.h
	@echo "Building unit test: $@"
	@$(CC) $(CFLAGS_TEST) $(INCLUDES) -o $@ $< $(CORE_SRCS) $(LIBS)

# Build integration tests
$(INTEGRATION_TEST_DIR)/%: $(INTEGRATION_TEST_DIR)/%.c vex.h
	@echo "Building integration test: $@"
	@$(CC) $(CFLAGS_TEST) $(INCLUDES) -o $@ $< $(CORE_SRCS) $(LIBS)

# Special case for INTEGRATION_DEMO (needs all libs)
$(INTEGRATION_TEST_DIR)/INTEGRATION_DEMO: $(INTEGRATION_TEST_DIR)/INTEGRATION_DEMO.c $(VEXTIME_LIB) $(VEXNET_LIB)
	@echo "Building INTEGRATION_DEMO..."
	@$(MAKE) -f Makefile.integration integration_demo
	@mv integration_demo $(INTEGRATION_TEST_DIR)/INTEGRATION_DEMO

# Test targets
test: test-unit test-integration
	@echo ""
	@echo "═══════════════════════════════════════════════════════════"
	@echo "  ✅ All Tests Passed!"
	@echo "═══════════════════════════════════════════════════════════"
	@echo ""

test-unit: $(UNIT_TESTS)
	@echo ""
	@echo "═══════════════════════════════════════════════════════════"
	@echo "  🧪 Running Unit Tests"
	@echo "═══════════════════════════════════════════════════════════"
	@echo ""
	@for test in $(UNIT_TESTS); do \
		if [ -f "$$test" ]; then \
			echo "Running $$test..."; \
			$$test || echo "❌ $$test failed"; \
			echo ""; \
		fi; \
	done
	@echo "✅ Unit tests complete"

test-integration: $(INTEGRATION_TESTS)
	@echo ""
	@echo "═══════════════════════════════════════════════════════════"
	@echo "  🔗 Running Integration Tests"
	@echo "═══════════════════════════════════════════════════════════"
	@echo ""
	@for test in $(INTEGRATION_TESTS); do \
		if [ -f "$$test" ] && [ -x "$$test" ]; then \
			echo "Running $$test..."; \
			$$test || echo "❌ $$test failed"; \
			echo ""; \
		fi; \
	done
	@echo "✅ Integration tests complete"

# Component-specific test shortcuts
test-vextime:
	@cd vex_time && $(MAKE) test

test-swisstable:
	@cd swisstable && ./build_swisstable.sh && ./vex_swisstable_test

test-async:
	@cd async_runtime && $(MAKE) test

# Benchmark targets
bench-swisstable:
	@cd swisstable && ./build_swisstable.sh && ./bench_ultimate

bench-vextime:
	@cd vex_time && $(MAKE) bench

# Clean
clean:
	@echo "Cleaning build artifacts..."
	@rm -f $(UNIT_TESTS)
	@rm -f $(INTEGRATION_TESTS)
	@rm -f $(UNIT_TEST_DIR)/*.o
	@rm -f $(INTEGRATION_TEST_DIR)/*.o
	@rm -f $(BUILD_DIR)/*
	@cd vex_time && $(MAKE) clean 2>/dev/null || true
	@cd vex_net && $(MAKE) clean 2>/dev/null || true
	@cd async_runtime && $(MAKE) clean 2>/dev/null || true
	@rm -f swisstable/*.o swisstable/*.a swisstable/vex_swisstable_test
	@rm -f swisstable/bench_* swisstable/insert_analysis
	@echo "✅ Clean complete"

# Info target
info:
	@echo "Vex Runtime C Library"
	@echo "====================="
	@echo ""
	@echo "Compiler: $(CC)"
	@echo "Flags: $(CFLAGS)"
	@echo ""
	@echo "Components:"
	@echo "  - vex_time: $(VEXTIME_LIB)"
	@echo "  - vex_net: $(VEXNET_LIB)"
	@echo "  - swisstable: $(SWISSTABLE_LIB)"
	@echo ""
	@echo "Tests:"
	@echo "  - Unit tests: $(words $(wildcard $(UNIT_TEST_DIR)/*.c))"
	@echo "  - Integration tests: $(words $(wildcard $(INTEGRATION_TEST_DIR)/*.c))"
	@echo ""

