cmake_minimum_required(VERSION 3.12)
project(vexnet C)
set(CMAKE_C_STANDARD 11)

add_library(vexnet STATIC
  src/socket_ops.c
  src/dns_dialer.c
  src/proxy_helpers.c
)

if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
  target_sources(vexnet PRIVATE src/backends/epoll.c)
elseif(CMAKE_SYSTEM_NAME STREQUAL "Darwin" OR CMAKE_SYSTEM_NAME MATCHES "BSD")
  target_sources(vexnet PRIVATE src/backends/kqueue.c)
elseif(WIN32)
  target_sources(vexnet PRIVATE src/backends/iocp.c)
endif()

target_include_directories(vexnet PUBLIC include)

add_executable(echo_server examples/echo_server.c)
target_link_libraries(echo_server vexnet)

add_executable(client_connect examples/client_connect.c)
target_link_libraries(client_connect vexnet)
