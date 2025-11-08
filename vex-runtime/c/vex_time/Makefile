CC ?= cc
CFLAGS ?= -O3 -Wall -Wextra -std=c11 -fPIC
INCLUDES := -Iinclude
LIBS := -pthread

# SIMD flags (auto-detect or specify)
SIMD_FLAGS ?= -march=native

SRCS_COMMON := src/common/vex_time_common.c src/common/tz_and_windows.c src/common/fast_parse.c src/common/layout_parse.c src/common/layout_format.c
SRCS_SIMD := src/common/simd_detect.c src/common/simd_rfc3339.c
SRCS_POSIX  := src/posix/time_posix.c
SRCS_WIN    := src/win/time_win.c
SRCS_LINUX_UR := src/linux/time_uring.c

OBJS_COMMON := $(SRCS_COMMON:.c=.o)
OBJS_SIMD := $(SRCS_SIMD:.c=.o)
OBJS_POSIX  := $(SRCS_POSIX:.c=.o)
OBJS_WIN    := $(SRCS_WIN:.c=.o)

all:
	@uname -s >/dev/null 2>&1 && $(MAKE) posix || $(MAKE) win

posix: libvextime.a examples
	@true

win: libvextime.a examples
	@true

libvextime.a: $(OBJS_COMMON) $(OBJS_SIMD) $(OBJS_POSIX)
	$(AR) rcs $@ $^

libvextime_win.a: $(OBJS_COMMON) $(OBJS_SIMD) $(OBJS_WIN)
	$(AR) rcs libvextime.a $^

src/common/simd_%.o: src/common/simd_%.c
	@mkdir -p $(@D)
	$(CC) $(CFLAGS) $(SIMD_FLAGS) $(INCLUDES) -c -o $@ $<

src/%.o: src/%.c include/vex_time.h
	@mkdir -p $(@D)
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

examples/%.o: examples/%.c include/vex_time.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

%.o: %.c include/vex_time.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

examples: examples/full_demo

examples/full_demo: examples/full_demo.o libvextime.a
	$(CC) $(CFLAGS) -o $@ examples/full_demo.o libvextime.a $(LIBS)

# Stress test
stress_test: stress_test.o libvextime.a
	$(CC) $(CFLAGS) -o $@ stress_test.o libvextime.a $(LIBS)

stress_test.o: stress_test.c include/vex_time.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

# SIMD benchmark
simd_bench: simd_bench.o libvextime.a
	$(CC) $(CFLAGS) -o $@ simd_bench.o libvextime.a $(LIBS)

simd_bench.o: simd_bench.c include/vex_time.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

# SWAR benchmark
swar_bench: swar_bench.o libvextime.a
	$(CC) $(CFLAGS) -o $@ swar_bench.o libvextime.a $(LIBS)

swar_bench.o: swar_bench.c include/vex_time.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

# Optional: io_uring timer scheduler (Linux)
uring:
	$(CC) $(CFLAGS) -DVEX_TIME_HAVE_URING $(INCLUDES) -c -o src/linux/time_uring.o src/linux/time_uring.c
	$(AR) rcs libvextime.a src/linux/time_uring.o $(SRCS_COMMON:.c=.o) $(SRCS_POSIX:.c=.o)

# Layout tests
layout_test: layout_test.o libvextime.a
	$(CC) $(CFLAGS) -o $@ layout_test.o libvextime.a $(LIBS)

layout_test.o: layout_test.c include/vex_time.h include/vex_time_layout.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

clean:
	rm -f src/common/*.o src/posix/*.o src/win/*.o src/linux/*.o 
	rm -f examples/*.o examples/full_demo
	rm -f stress_test.o stress_test
	rm -f simd_bench.o simd_bench
	rm -f swar_bench.o swar_bench
	rm -f layout_test.o layout_test
	rm -f libvextime.a

test: stress_test
	@echo ""
	@echo "Running stress test..."
	@./stress_test

bench: simd_bench
	@echo ""
	@echo "Running SIMD benchmark..."
	@./simd_bench

swar: swar_bench
	@echo ""
	@echo "Running SWAR benchmark..."
	@./swar_bench

layout: layout_test
	@echo ""
	@echo "Running layout tests..."
	@./layout_test

# Build with specific SIMD levels
avx2:
	$(MAKE) clean
	$(MAKE) SIMD_FLAGS="-mavx2" all simd_bench
	@echo ""
	@echo "✅ Built with AVX2 support"

avx512:
	$(MAKE) clean
	$(MAKE) SIMD_FLAGS="-mavx512f" all simd_bench
	@echo ""
	@echo "✅ Built with AVX-512 support"

native:
	$(MAKE) clean
	$(MAKE) SIMD_FLAGS="-march=native" all simd_bench
	@echo ""
	@echo "✅ Built with native CPU optimizations"

.PHONY: all posix win examples clean uring test bench avx2 avx512 native layout
