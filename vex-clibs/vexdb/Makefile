CC ?= cc
AR ?= ar
CFLAGS ?= -O3 -Wall -Wextra -std=c11 -fPIC
INCLUDES := -Iinclude
LIBS :=
OBJS := src/vex_pg.o src/vex_mongo.o
DEMO_OBJS := src/vex_db_demo.o

# Enable libpq support with: make HAVE_LIBPQ=1 PQ_CFLAGS="$(pkg-config --cflags libpq)" PQ_LDFLAGS="$(pkg-config --libs libpq)"
ifeq ($(HAVE_LIBPQ),1)
CFLAGS += -DHAVE_LIBPQ $(PQ_CFLAGS)
LIBS += $(PQ_LDFLAGS)
endif

all: libvexdb.a vex_db_demo

libvexdb.a: $(OBJS)
	$(AR) rcs $@ $(OBJS)

vex_db_demo: $(DEMO_OBJS) libvexdb.a
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ $(DEMO_OBJS) libvexdb.a $(LIBS)

clean:
	rm -f $(OBJS) $(DEMO_OBJS) libvexdb.a vex_db_demo

.PHONY: all clean
