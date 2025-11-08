cmake_minimum_required(VERSION 3.12)
project(vextime C)
set(CMAKE_C_STANDARD 11)

add_library(vextime STATIC
  src/vex_time_common.c
)

if(WIN32)
  target_sources(vextime PRIVATE src/vex_time_win.c)
else()
  target_sources(vextime PRIVATE src/vex_time_posix.c)
endif()

target_include_directories(vextime PUBLIC include)

add_executable(time_demo examples/time_demo.c)
target_link_libraries(time_demo vextime)
