CC = gcc
CFLAGS = -std=c11 -O2 -Wall -Wextra -pthread
UNAME_S := $(shell uname -s)
KERNEL_R := $(shell uname -r)
KERNEL_MAJ := $(word 1, $(subst ., ,$(KERNEL_R)))
KERNEL_MIN := $(word 2, $(subst ., ,$(KERNEL_R)))

# default poller per-OS
ifeq ($(UNAME_S),Linux)
    # decide io_uring vs epoll
    # if kernel >= 5.11 -> io_uring, else epoll
    CMP_GE_5_11 := $(shell [ $(KERNEL_MAJ) -gt 5 -o \( $(KERNEL_MAJ) -eq 5 -a $(KERNEL_MIN) -ge 11 \) ] && echo yes || echo no)
    ifeq ($(CMP_GE_5_11),yes)
        POLLER ?= io_uring
    else
        POLLER ?= epoll
    endif
endif

ifeq ($(UNAME_S),Darwin)
    POLLER ?= kqueue
endif

ifeq ($(UNAME_S),FreeBSD)
    POLLER ?= kqueue
endif

ifeq ($(OS),Windows_NT)
    POLLER ?= iocp
endif

SRC_COMMON = src/runtime.c src/worker_context.c src/lockfree_queue.c src/common.c

ifeq ($(POLLER),io_uring)
    SRC_POLL = src/poller_io_uring.c
    LIBS = -luring
endif
ifeq ($(POLLER),epoll)
    SRC_POLL = src/poller_epoll.c
endif
ifeq ($(POLLER),kqueue)
    SRC_POLL = src/poller_kqueue.c
endif
ifeq ($(POLLER),iocp)
    SRC_POLL = src/poller_iocp.c
endif

SRC = $(SRC_COMMON) $(SRC_POLL)
INC = -Iinclude
OBJ = $(SRC:.c=.o)
TARGET = async_runtime_demo

all: $(TARGET)

$(TARGET): $(OBJ) tests/example_async_demo.o
	$(CC) $(CFLAGS) -o $@ $^ $(LIBS)

tests/example_async_demo.o: tests/example_async_demo.c include/runtime.h include/poller.h
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

%.o: %.c
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

clean:
	rm -f $(OBJ) tests/example_async_demo.o $(TARGET)

print-config:
	@echo OS=$(UNAME_S)
	@echo KERNEL=$(KERNEL_R)
	@echo POLLER=$(POLLER)
