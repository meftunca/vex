# vex_net Benchmarks Makefile
# High-performance network benchmarking suite

CC ?= cc
CFLAGS ?= -O3 -Wall -Wextra -std=c11
INCLUDES := -I../include
LIBS :=

# Platform detection
UNAME_S := $(shell uname -s)

# Backend selection
ifeq ($(UNAME_S),Linux)
  ifneq ($(USE_IOURING),)
    SRCS_BACKEND := ../src/backends/io_uring.c
    CFLAGS += -DVEX_NET_HAVE_IOURING
    LIBS += -luring
  else
    SRCS_BACKEND := ../src/backends/epoll.c
  endif
endif

ifeq ($(UNAME_S),Darwin)
  SRCS_BACKEND := ../src/backends/kqueue.c
endif

ifeq ($(UNAME_S),FreeBSD)
  SRCS_BACKEND := ../src/backends/kqueue.c
endif

# Source files
SRCS := ../src/socket_ops.c ../src/dns_dialer.c ../src/proxy_helpers.c $(SRCS_BACKEND)
OBJS := $(SRCS:.c=.o)

# Benchmarks
BENCHMARKS := benchmark_echo benchmark_client

all: $(BENCHMARKS)

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

# Echo server benchmark
benchmark_echo: benchmark_echo.o vex_alloc_stub.o $(OBJS)
	$(CC) $(CFLAGS) -o $@ benchmark_echo.o vex_alloc_stub.o $(OBJS) $(LIBS)

# Client load generator
benchmark_client: benchmark_client.o vex_alloc_stub.o $(OBJS)
	$(CC) $(CFLAGS) -o $@ benchmark_client.o vex_alloc_stub.o $(OBJS) $(LIBS)

# Run benchmarks
test: all
	@echo "═══════════════════════════════════════════════════════════"
	@echo "  Running vex_net Performance Benchmarks"
	@echo "═══════════════════════════════════════════════════════════"
	@echo ""
	@echo "Starting echo server in background..."
	@./benchmark_echo & echo $$! > .server.pid
	@sleep 2
	@echo ""
	@echo "Running client load generator..."
	@./benchmark_client || true
	@echo ""
	@echo "Stopping server..."
	@kill `cat .server.pid` 2>/dev/null || true
	@rm -f .server.pid
	@echo ""
	@echo "✅ Benchmark complete!"

clean:
	rm -f *.o $(BENCHMARKS) .server.pid
	rm -f ../src/*.o ../src/backends/*.o

.PHONY: all test clean
