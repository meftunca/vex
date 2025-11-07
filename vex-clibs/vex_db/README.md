
# VexDB — Universal Database Driver for Vex

**Production-ready** — 5 database drivers with zero-copy interfaces.

## Supported Databases

| Database | Driver | Status | Type |
|----------|--------|--------|------|
| **PostgreSQL** | libpq | ✅ Full | SQL |
| **MySQL/MariaDB** | libmysqlclient | ✅ Full | SQL |
| **SQLite** | sqlite3 | ✅ Full | SQL |
| **MongoDB** | libmongoc | ✅ Full | NoSQL (Document) |
| **Redis** | hiredis | ✅ Full | NoSQL (Key-Value) |

## Quick Start

### SQLite (Default)
```bash
make
./demo sqlite ./test.db "SELECT 1 as one"
```

### Build with All Drivers
```bash
make clean
make HAVE_SQLITE=1 HAVE_LIBPQ=1 HAVE_MYSQL=1 HAVE_MONGO=1 HAVE_REDIS=1 \
  PQ_CFLAGS="$(pkg-config --cflags libpq)" \
  PQ_LDFLAGS="$(pkg-config --libs libpq)" \
  MYSQL_CFLAGS="$(mysql_config --cflags)" \
  MYSQL_LDFLAGS="$(mysql_config --libs)" \
  MONGO_CFLAGS="$(pkg-config --cflags libmongoc-1.0)" \
  MONGO_LDFLAGS="$(pkg-config --libs libmongoc-1.0)" \
  REDIS_CFLAGS="" \
  REDIS_LDFLAGS="-lhiredis"
```

## Driver-Specific Examples

### PostgreSQL
```bash
./demo postgres "host=localhost user=postgres" "SELECT version()"
```

### MySQL
```bash
./demo mysql "host=localhost user=root password=secret db=test" "SELECT 1"
```

### SQLite
```bash
./demo sqlite ./test.db "SELECT datetime('now')"
```

### MongoDB
```bash
# Find all documents in 'users' collection
./demo mongo "mongodb://localhost:27017/mydb" "users.find({})"

# Find with query
./demo mongo "mongodb://localhost:27017/mydb" "users.find({\"age\":25})"

# Aggregate
./demo mongo "mongodb://localhost:27017/mydb" "users.aggregate([{\"$match\":{}}])"
```

### Redis
```bash
# Get value
./demo redis "localhost:6379" "GET mykey"

# Set value (use with parameters in real code)
./demo redis "localhost:6379" "SET mykey myvalue"

# List range
./demo redis "localhost:6379" "LRANGE mylist 0 -1"

# Unix socket
./demo redis "/tmp/redis.sock" "PING"
```

## Build Individual Drivers

### PostgreSQL Only
```bash
make HAVE_SQLITE=0 HAVE_LIBPQ=1 \
  PQ_CFLAGS="$(pkg-config --cflags libpq)" \
  PQ_LDFLAGS="$(pkg-config --libs libpq)"
```

### MongoDB Only
```bash
make HAVE_SQLITE=0 HAVE_MONGO=1 \
  MONGO_CFLAGS="$(pkg-config --cflags libmongoc-1.0)" \
  MONGO_LDFLAGS="$(pkg-config --libs libmongoc-1.0)"
```

### Redis Only
```bash
make HAVE_SQLITE=0 HAVE_REDIS=1 \
  REDIS_LDFLAGS="-lhiredis"
```

## Features

### Zero-Copy Interface
- `fetch_next` exposes **first column** zero-copy for speed
- Use `column_*` metadata for multi-column coordination on Vex side

### Memory Lifetimes
- **PostgreSQL**: `VEX_LIFE_RESULT_OWNED` (valid until `clear_result`)
- **MySQL**: `VEX_LIFE_ROW_BUFFER` (valid until next `mysql_fetch_row`)
- **SQLite**: `VEX_LIFE_ROW_BUFFER` (valid until next `sqlite3_step`)
- **MongoDB**: `VEX_LIFE_ROW_BUFFER` (valid until next document fetch)
- **Redis**: `VEX_LIFE_RESULT_OWNED` (valid until `clear_result`)

### Advanced Features

#### Pub/Sub Support
- **PostgreSQL**: Full LISTEN/NOTIFY support for real-time events
- **Redis**: Complete Pub/Sub implementation (SUBSCRIBE/PUBLISH)
- Event-driven architecture with `poll_notifications()` and `get_notification()`

#### Transaction Support
- **PostgreSQL**: Full ACID transactions (BEGIN/COMMIT/ROLLBACK)
- **MySQL**: Transaction support (START TRANSACTION/COMMIT/ROLLBACK)
- **SQLite**: Transaction support (BEGIN/COMMIT/ROLLBACK)
- **Redis**: MULTI/EXEC/DISCARD for atomic operations

#### Async Query Execution
- **PostgreSQL**: Full async query support with event FD polling
- Non-blocking I/O for high-performance applications
- `start_execute()`, `poll_ready()`, `get_result()` pattern

### Type System
- `VEX_T_NULL` - NULL value
- `VEX_T_BOOL` - Boolean
- `VEX_T_I64` - 64-bit integer
- `VEX_T_F64` - 64-bit float
- `VEX_T_TEXT` - Text string
- `VEX_T_BIN` - Binary data
- `VEX_T_JSON` - JSON document (MongoDB)

## Query Formats

### SQL Databases (PostgreSQL, MySQL, SQLite)
Standard SQL syntax:
```sql
SELECT * FROM users WHERE age > 21
INSERT INTO users (name, age) VALUES ('Alice', 25)
```

### MongoDB
Collection operation format:
```
collection.operation(query)
```

Operations:
- `find({...})` - Find documents
- `aggregate([...])` - Aggregation pipeline

### Redis
Standard Redis commands:
```
GET key
SET key value
HGETALL myhash
LRANGE mylist 0 -1
```

## Dependencies

Install required libraries:

```bash
# Ubuntu/Debian
sudo apt-get install \
  libpq-dev \
  libmysqlclient-dev \
  libsqlite3-dev \
  libmongoc-dev \
  libhiredis-dev

# macOS (Homebrew)
brew install \
  postgresql \
  mysql-client \
  sqlite \
  mongo-c-driver \
  hiredis

# Fedora/RHEL
sudo dnf install \
  libpq-devel \
  mysql-devel \
  sqlite-devel \
  mongo-c-driver-devel \
  hiredis-devel
```

## Architecture

```
VexDbDriver (Interface)
    ├── VEX_DRIVER_POSTGRES  (libpq)
    ├── VEX_DRIVER_MYSQL     (libmysqlclient)
    ├── VEX_DRIVER_SQLITE    (sqlite3)
    ├── VEX_DRIVER_MONGO     (libmongoc)
    └── VEX_DRIVER_REDIS     (hiredis)
```

## Advanced Usage Examples

### PostgreSQL LISTEN/NOTIFY
```c
VexConnection conn = VEX_DRIVER_POSTGRES.connect("host=localhost");

// Subscribe to a channel
VEX_DRIVER_POSTGRES.subscribe(&conn, "notifications");

// In another connection, publish
VEX_DRIVER_POSTGRES.publish(&conn2, "notifications", "Hello!", 6);

// Poll for notifications
if (VEX_DRIVER_POSTGRES.poll_notifications(&conn) > 0) {
    VexDbPayload notif = VEX_DRIVER_POSTGRES.get_notification(&conn);
    printf("Received: %.*s\n", (int)notif.length, notif.data);
}
```

### Redis Pub/Sub
```c
VexConnection conn = VEX_DRIVER_REDIS.connect("localhost:6379");

// Subscribe to a channel
VEX_DRIVER_REDIS.subscribe(&conn, "events");

// Publish (returns number of subscribers)
int subscribers = VEX_DRIVER_REDIS.publish(&conn2, "events", "Update!", 7);

// Get notification
VexDbPayload notif = VEX_DRIVER_REDIS.get_notification(&conn);
```

### Transactions
```c
VexConnection conn = VEX_DRIVER_POSTGRES.connect("...");

// Start transaction
VEX_DRIVER_POSTGRES.begin_transaction(&conn);

// Execute queries
VEX_DRIVER_POSTGRES.execute_query(&conn, "INSERT INTO ...", 0, NULL);
VEX_DRIVER_POSTGRES.execute_query(&conn, "UPDATE ...", 0, NULL);

// Commit or rollback
if (success) {
    VEX_DRIVER_POSTGRES.commit_transaction(&conn);
} else {
    VEX_DRIVER_POSTGRES.rollback_transaction(&conn);
}
```

### Async Queries (PostgreSQL)
```c
VexConnection conn = VEX_DRIVER_POSTGRES.connect("...");

// Start async query
VEX_DRIVER_POSTGRES.start_execute(&conn, "SELECT * FROM large_table", 0, NULL);

// Poll until ready
int fd = VEX_DRIVER_POSTGRES.get_event_fd(&conn);
// Use select/poll/epoll on fd

while (VEX_DRIVER_POSTGRES.poll_ready(&conn) == 0) {
    // Wait for data
}

// Get result
VexResultSet rs = VEX_DRIVER_POSTGRES.get_result(&conn);
```

## Testing

### Quick Test (SQLite only)
```bash
make test
```

### Full Integration Test (Docker required)
```bash
# Start all databases and run comprehensive tests
./run_tests.sh
```

### Manual Docker Setup
```bash
# Start databases
docker-compose up -d

# Wait for databases to be ready (5-10 seconds)
sleep 10

# Build with all drivers
make clean
make HAVE_LIBPQ=1 HAVE_MYSQL=1 HAVE_MONGO=1 HAVE_REDIS=1 HAVE_SQLITE=1 \
  PQ_CFLAGS="$(pkg-config --cflags libpq)" \
  PQ_LDFLAGS="$(pkg-config --libs libpq)" \
  MYSQL_CFLAGS="$(mysql_config --cflags)" \
  MYSQL_LDFLAGS="$(mysql_config --libs)" \
  MONGO_CFLAGS="$(pkg-config --cflags libmongoc-1.0)" \
  MONGO_LDFLAGS="$(pkg-config --libs libmongoc-1.0)" \
  REDIS_LDFLAGS="-lhiredis" \
  test_integration

# Run tests
./test_integration

# Cleanup
docker-compose down
```

## Version

**v2.2.0** - Cursor Streaming & Comprehensive Testing

### Changelog

#### v2.2.0 (November 2025)
- ✅ Added cursor-based streaming for PostgreSQL (`declare_cursor`, `fetch_from_cursor`, `close_cursor`)
- ✅ Added Docker Compose setup for all databases
- ✅ Added comprehensive integration test suite
- ✅ Added automated test runner script (`run_tests.sh`)
- ✅ Full CRUD, transaction, Pub/Sub, and streaming tests
- ✅ Production-ready with real-world testing

#### v2.1.0 (November 2025)
- ✅ Added PostgreSQL LISTEN/NOTIFY support
- ✅ Added Redis Pub/Sub support  
- ✅ Added transaction support (PostgreSQL, MySQL, SQLite, Redis)
- ✅ Enhanced async query support (PostgreSQL)
- ✅ Added `VEX_CAP_PUBSUB` and `VEX_CAP_TXN` capability flags
- ✅ Comprehensive API for real-time and transactional applications

#### v2.0.0 (November 2025)
- ✅ Added MongoDB driver (libmongoc)
- ✅ Added Redis driver (hiredis)
- ✅ Enhanced demo with better output formatting
- ✅ Conditional compilation for all drivers
- ✅ Comprehensive documentation

#### v1.0.0
- ✅ Initial release (PostgreSQL, MySQL, SQLite)

## License
MIT
