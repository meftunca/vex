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

SRCS := src/socket_ops.c src/dns_dialer.c src/proxy_helpers.c $(SRCS_BACKEND)

all: libvexnet.a examples

libvexnet.a: $(SRCS:.c=.o)
	$(AR) rcs $@ $^

%.o: %.c include/vex_net.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

examples: examples/echo_server examples/client_connect

examples/echo_server: examples/echo_server.o libvexnet.a
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ examples/echo_server.o libvexnet.a $(LIBS)

examples/client_connect: examples/client_connect.o libvexnet.a
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ examples/client_connect.o libvexnet.a $(LIBS)

clean:
	rm -f src/*.o src/backends/*.o examples/*.o libvexnet.a examples/echo_server examples/client_connect
