CC ?= cc
CFLAGS ?= -O3 -Wall -Wextra -std=c11 -fPIC
INCLUDES := -Iinclude
LIBS :=

ifeq ($(shell uname -s),Linux)
  # Check for io_uring support
  ifneq ($(USE_IOURING),)
    SRCS_BACKEND := src/backends/io_uring.c
    CFLAGS += -DVEX_NET_HAVE_URING
    LIBS += -luring
  else
    SRCS_BACKEND := src/backends/epoll.c
  endif
endif
ifeq ($(shell uname -s),Darwin)
  SRCS_BACKEND := src/backends/kqueue.c
endif
ifeq ($(shell uname -s),FreeBSD)
  SRCS_BACKEND := src/backends/kqueue.c
endif

SRCS_PROTOCOLS := src/protocols/simd_utils.c \
                   src/protocols/http_parser.c \
                   src/protocols/http2_parser.c \
                   src/protocols/hpack.c \
                   src/protocols/websocket_parser.c \
                   src/protocols/dns_parser.c \
                   src/protocols/tls_detector.c \
                   src/protocols/icmp_parser.c \
                   src/protocols/udp_parser.c

SRCS := src/socket_ops.c src/dns_dialer.c src/proxy_helpers.c $(SRCS_BACKEND) $(SRCS_PROTOCOLS)

all: libvexnet.a examples

libvexnet.a: $(SRCS:.c=.o)
	$(AR) rcs $@ $^

%.o: %.c include/vex_net.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

examples: examples/echo_server examples/client_connect examples/protocol_demo

# Examples
examples/echo_server: examples/echo_server.o libvexnet.a
	$(CC) $(CFLAGS) -o $@ $^

examples/client_connect: examples/client_connect.o libvexnet.a
	$(CC) $(CFLAGS) -o $@ $^

examples/protocol_demo: examples/protocol_demo.o libvexnet.a
	$(CC) $(CFLAGS) -o $@ $^

# Tests
tests/protocol_test_runner: tests/protocols/protocol_test_runner.o libvexnet.a
	$(CC) $(CFLAGS) -o $@ $^

# Object file rule for test runner
tests/protocols/protocol_test_runner.o: tests/protocols/protocol_test_runner.c
	$(CC) $(CFLAGS) -Iinclude -c -o $@ $<

clean:
	rm -f src/*.o src/backends/*.o src/protocols/*.o examples/*.o libvexnet.a examples/echo_server examples/client_connect examples/protocol_demo tests/protocols/*.o tests/protocol_test_runner
