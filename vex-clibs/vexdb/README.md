# Vex Universal DB Driver (C) — Minimal Integration Pack

This pack provides a **minimal, Vex-integratable** C driver interface with:
- Zero-copy/zero-allocation (as much as the native client allows)
- Atomic functions (`connect`, `execute_query`, `fetch_next`, `clear_result`)
- Capability flags (SQL/Doc/Async)
- **Asynchronous usage** commented inline using libpq/libmongoc patterns
- A PostgreSQL implementation that **compiles to a mock by default** and
  enables libpq-backed implementation if built with `-DHAVE_LIBPQ` and linked to libpq.
- A MongoDB mock that shows the shape for a real libmongoc wrapper

> You can wire this directly into your Vex FFI. The header is stable and small.

## Layout
```
include/vex_db_driver.h
src/vex_pg.c
src/vex_mongo.c
src/vex_db_demo.c
Makefile
```

## Build (mock drivers only, no external deps)
```bash
make           # builds libvexdb.a and demo
./vex_db_demo
```

## Build with libpq (PostgreSQL)
```bash
make clean
make HAVE_LIBPQ=1 PQ_CFLAGS="$(pkg-config --cflags libpq)" PQ_LDFLAGS="$(pkg-config --libs libpq)"
# example run (adjust conninfo and query)
./vex_db_demo "host=127.0.0.1 user=postgres dbname=postgres" "select 1"
```

## Integrating with Vex
- Bind the C functions via your FFI (stable C ABI).
- Use `execute_query` + `fetch_next` for a **pull iterator**.
- To expose **channels** at the Vex level, spawn workers that call `fetch_next`
  and push rows/documents into bounded channels (see comments in `vex_db_demo.c`).

## License
MIT.
