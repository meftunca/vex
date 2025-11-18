CC ?= cc
CFLAGS ?= -O3 -Wall -Wextra -std=c11 -fPIC
INCLUDES := -Iinclude

SRCS_COMMON := src/common/hex_scalar.c src/common/base64_scalar.c src/common/base32_scalar.c \
               src/common/uuid_all.c src/util/os_random.c src/util/md5.c src/util/sha1.c src/util/cpufeatures.c

SRCS_X86 := src/x86/hex_avx2.c src/x86/hex_avx512.c
SRCS_ARM := src/arm/hex_neon.c

OBJS := $(SRCS_COMMON:.c=.o)

all: libvexfastenc.a tests bench

%.o: %.c include/vex_fastenc.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

libvexfastenc.a: $(OBJS)
	@# Conditionally compile arch-specific files
	@ARCH_OBJS=""; \
	if echo | $(CC) -E -dM -xc - | grep -q "__x86_64__"; then \
	  echo "Building x86-64 SIMD (AVX2/AVX512)..."; \
	  $(CC) $(CFLAGS) $(INCLUDES) -c -o src/x86/hex_avx2.o src/x86/hex_avx2.c -mavx2 || true; \
	  $(CC) $(CFLAGS) $(INCLUDES) -c -o src/x86/hex_avx512.o src/x86/hex_avx512.c -mavx512bw -mavx512f || true; \
	  ARCH_OBJS="src/x86/hex_avx2.o src/x86/hex_avx512.o"; \
	elif echo | $(CC) -E -dM -xc - | grep -q "__aarch64__"; then \
	  echo "Building ARM64 SIMD (NEON)..."; \
	  $(CC) $(CFLAGS) $(INCLUDES) -c -o src/arm/hex_neon.o src/arm/hex_neon.c || true; \
	  ARCH_OBJS="src/arm/hex_neon.o"; \
	fi; \
	ar rcs $@ $(OBJS) $$ARCH_OBJS

tests: libvexfastenc.a tests/test_vectors.o
	$(CC) $(CFLAGS) $(INCLUDES) -o tests/test_vectors tests/test_vectors.o libvexfastenc.a

bench: libvexfastenc.a bench/bench.o
	$(CC) $(CFLAGS) $(INCLUDES) -o bench/bench bench/bench.o libvexfastenc.a

clean:
	rm -f $(SRCS_COMMON:.c=.o) src/x86/*.o src/arm/*.o tests/*.o bench/*.o libvexfastenc.a tests/test_vectors bench/bench
