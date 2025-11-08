CC ?= cc
CFLAGS ?= -O3 -Wall -Wextra -std=c11 -fPIC
INCLUDES := -Iinclude

SRCS_POSIX := src/vex_time_common.c src/vex_time_posix.c
SRCS_WIN   := src/vex_time_common.c src/vex_time_win.c

all:
	@uname -s >/dev/null 2>&1 && $(MAKE) posix || $(MAKE) win

posix: libvextime.a examples
	@true

win: libvextime.a examples
	@true

libvextime.a:
	@uname -s >/dev/null 2>&1 && $(AR) rcs $@ $(SRCS_POSIX:.c=.o) || $(AR) rcs $@ $(SRCS_WIN:.c=.o)

src/%.o: src/%.c include/vex_time.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

examples: examples/time_demo

examples/time_demo: examples/time_demo.o libvextime.a
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ examples/time_demo.o libvextime.a

clean:
	rm -f src/*.o libvextime.a examples/time_demo
