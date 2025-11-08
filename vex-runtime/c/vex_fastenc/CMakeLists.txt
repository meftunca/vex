cmake_minimum_required(VERSION 3.12)
project(vex_fastenc C)
set(CMAKE_C_STANDARD 11)

add_library(vexfastenc STATIC
  src/common/hex_scalar.c
  src/common/base64_scalar.c
  src/common/base32_scalar.c
  src/common/uuid_all.c
  src/util/os_random.c
  src/util/md5.c
  src/util/sha1.c
  src/util/cpufeatures.c
)

target_include_directories(vexfastenc PUBLIC include)

if (CMAKE_SYSTEM_PROCESSOR MATCHES "x86_64|AMD64|i.86")
  add_library(vexfastenc_x86 OBJECT src/x86/hex_avx2.c src/x86/hex_avx512.c)
  target_compile_options(vexfastenc_x86 PRIVATE -mavx2 -mavx512f -mavx512bw)
  target_link_libraries(vexfastenc PUBLIC vexfastenc_x86)
endif()

if (CMAKE_SYSTEM_PROCESSOR MATCHES "aarch64|arm64|ARM64|ARM")
  add_library(vexfastenc_arm OBJECT src/arm/hex_neon.c)
  target_link_libraries(vexfastenc PUBLIC vexfastenc_arm)
endif()

add_executable(test_vectors tests/test_vectors.c)
target_link_libraries(test_vectors vexfastenc)

add_executable(bench bench/bench.c)
target_link_libraries(bench vexfastenc)
