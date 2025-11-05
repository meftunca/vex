CC ?= clang
CFLAGS ?= -std=c11 -O2 -Wall -Wextra -fPIC
PKG ?= pkg-config

OPENSSL_CFLAGS := $(shell $(PKG)  --cflags openssl 2>/dev/null)
OPENSSL_LIBS   := $(shell $(PKG)  --libs openssl 2>/dev/null)

INC = -Iinclude $(OPENSSL_CFLAGS)
LIBS = $(OPENSSL_LIBS)

OBJS_TLS = src/vex_tls_openssl.o
OBJS_CRYPTO = src/vex_crypto_openssl.o

all: libvex_tls.a libvex_crypto.a test_crypto

libvex_tls.a: $(OBJS_TLS)
	ar rcs $@ $^

libvex_crypto.a: $(OBJS_CRYPTO)
	ar rcs $@ $^

src/%.o: src/%.c include/vex_tls.h include/vex_crypto.h
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

tests/test_crypto: tests/test_crypto.o libvex_crypto.a
	$(CC) $(CFLAGS) -o $@ $^ $(LIBS)

tests/test_crypto.o: tests/test_crypto.c include/vex_crypto.h include/vex_tls.h
	$(CC) $(CFLAGS) $(INC) -c -o $@ $<

test_crypto: tests/test_crypto
	@./tests/test_crypto || true

clean:
	rm -f src/*.o tests/*.o *.a tests/test_crypto
