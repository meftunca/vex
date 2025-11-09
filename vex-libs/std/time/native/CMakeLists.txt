cmake_minimum_required(VERSION 3.12)
project(vextime_ultrafull C)
set(CMAKE_C_STANDARD 11)

add_library(vextime STATIC
  src/common/vex_time_common.c
  src/common/tz_and_windows.c
)

if(WIN32)
  target_sources(vextime PRIVATE src/win/time_win.c)
else()
  target_sources(vextime PRIVATE src/posix/time_posix.c)
endif()

target_include_directories(vextime PUBLIC include)

add_executable(full_demo examples/full_demo.c)
target_link_libraries(full_demo vextime)
