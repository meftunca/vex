
CC ?= cc
AR ?= ar
CFLAGS ?= -O3 -Wall -Wextra -std=c11 -fPIC
INCLUDES := -Iinclude
LIBS :=

# Toggle drivers via HAVE_* flags. Default enables SQLite only.
HAVE_LIBPQ ?= 0
HAVE_MYSQL ?= 0
HAVE_SQLITE ?= 1
HAVE_MONGO ?= 0
HAVE_REDIS ?= 0

OBJS :=

ifeq ($(HAVE_LIBPQ),1)
CFLAGS += -DHAVE_LIBPQ $(PQ_CFLAGS)
LIBS   += $(PQ_LDFLAGS)
OBJS   += src/vex_pg.o
endif

ifeq ($(HAVE_MYSQL),1)
CFLAGS += -DHAVE_MYSQL $(MYSQL_CFLAGS)
LIBS   += $(MYSQL_LDFLAGS)
OBJS   += src/vex_mysql.o
endif

ifeq ($(HAVE_SQLITE),1)
CFLAGS += -DHAVE_SQLITE $(SQLITE_CFLAGS)
SQLITE_LDFLAGS ?= -lsqlite3
LIBS   += $(SQLITE_LDFLAGS)
OBJS   += src/vex_sqlite.o
endif

ifeq ($(HAVE_MONGO),1)
CFLAGS += -DHAVE_MONGO $(MONGO_CFLAGS)
LIBS   += $(MONGO_LDFLAGS)
OBJS   += src/vex_mongo.o
endif

ifeq ($(HAVE_REDIS),1)
CFLAGS += -DHAVE_REDIS $(REDIS_CFLAGS)
LIBS   += $(REDIS_LDFLAGS)
OBJS   += src/vex_redis.o
endif

all: libvexdb.a demo

libvexdb.a: $(OBJS)
	$(AR) rcs $@ $(OBJS)

src/%.o: src/%.c include/vex_db_driver.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

examples/%.o: examples/%.c include/vex_db_driver.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

tests/%.o: tests/%.c include/vex_db_driver.h
	$(CC) $(CFLAGS) $(INCLUDES) -c -o $@ $<

demo: examples/demo.o libvexdb.a
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ examples/demo.o libvexdb.a $(LIBS)

test_all_drivers: tests/test_all_drivers.o libvexdb.a
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ tests/test_all_drivers.o libvexdb.a $(LIBS)

test_integration: tests/test_integration.o libvexdb.a
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ tests/test_integration.o libvexdb.a $(LIBS)

test: test_all_drivers
	./test_all_drivers

test-integration: test_integration
	./test_integration

test-all: test test-integration

clean:
	rm -f src/*.o examples/*.o tests/*.o libvexdb.a demo test_all_drivers test_integration

.PHONY: all clean test test-integration test-all
