# VexDB Testing Guide

## Overview

VexDB comes with comprehensive test suites to ensure reliability across all supported databases.

## Test Types

### 1. Unit Tests (`test_all_drivers`)

- Basic driver functionality
- SQLite-only (no external dependencies)
- Quick sanity checks
- Run with: `make test`

### 2. Integration Tests (`test_integration`)

- Real database connections via Docker
- Full CRUD operations
- Transaction testing
- Pub/Sub testing (PostgreSQL, Redis)
- Cursor streaming (PostgreSQL)
- Run with: `./run_tests.sh` or `make test-integration`

## Quick Start

### Option 1: Automated (Recommended)

```bash
./run_tests.sh
```

This script will:

1. Start Docker Compose with all databases
2. Wait for services to be healthy
3. Build VexDB with all drivers
4. Run comprehensive integration tests
5. Clean up containers (if tests pass)

### Option 2: Manual Setup

#### Step 1: Start Databases

```bash
docker-compose up -d
```

#### Step 2: Wait for Services

```bash
# Wait 10 seconds for all services to be ready
sleep 10

# Or check individual services:
docker-compose ps
docker-compose logs postgres
```

#### Step 3: Build with All Drivers

```bash
make clean
make HAVE_LIBPQ=1 HAVE_MYSQL=1 HAVE_MONGO=1 HAVE_REDIS=1 HAVE_SQLITE=1 \
  PQ_CFLAGS="$(pkg-config --cflags libpq)" \
  PQ_LDFLAGS="$(pkg-config --libs libpq)" \
  MYSQL_CFLAGS="$(mysql_config --cflags)" \
  MYSQL_LDFLAGS="$(mysql_config --libs)" \
  MONGO_CFLAGS="$(pkg-config --cflags libmongoc-1.0)" \
  MONGO_LDFLAGS="$(pkg-config --libs libmongoc-1.0)" \
  REDIS_LDFLAGS="-lhiredis"
```

#### Step 4: Run Tests

```bash
# Unit tests
make test

# Integration tests
./test_integration

# Or both
make test-all
```

#### Step 5: Cleanup

```bash
docker-compose down
```

## Database Connection Details

### PostgreSQL

- Host: `localhost`
- Port: `5432`
- User: `vexdb`
- Password: `vexdb_test`
- Database: `vexdb_test`

### MySQL

- Host: `localhost`
- Port: `3306`
- User: `vexdb`
- Password: `vexdb_test`
- Database: `vexdb_test`

### MongoDB

- Host: `localhost`
- Port: `27017`
- User: `vexdb`
- Password: `vexdb_test`
- Database: `vexdb_test`
- URI: `mongodb://vexdb:vexdb_test@localhost:27017/vexdb_test`

### Redis

- Host: `localhost`
- Port: `6379`
- No authentication

### SQLite

- In-memory database (`:memory:`)
- No external setup required

## Test Coverage

### PostgreSQL Tests

- ✅ Full CRUD operations (CREATE, INSERT, SELECT, UPDATE, DELETE)
- ✅ Transaction support (BEGIN, COMMIT, ROLLBACK)
- ✅ LISTEN/NOTIFY (Pub/Sub)
- ✅ Cursor-based streaming (DECLARE CURSOR, FETCH, CLOSE)
- ✅ Error handling
- ✅ NULL value handling

### MySQL Tests

- ✅ Full CRUD operations
- ✅ Transaction support (START TRANSACTION, COMMIT, ROLLBACK)
- ✅ Error handling

### SQLite Tests

- ✅ In-memory database operations
- ✅ Transaction support
- ✅ NULL value handling
- ✅ Error handling

### MongoDB Tests

- ✅ Basic find operations
- ✅ JSON document output
- ✅ Error handling

### Redis Tests

- ✅ GET/SET operations
- ✅ Transaction support (MULTI/EXEC/DISCARD)
- ✅ List operations (RPUSH, LRANGE)
- ✅ Error handling

## Troubleshooting

### Docker Issues

```bash
# Check if Docker is running
docker info

# View container logs
docker-compose logs <service>

# Restart containers
docker-compose restart

# Force recreate containers
docker-compose up -d --force-recreate
```

### Build Issues

```bash
# Install missing dependencies (macOS with Homebrew)
brew install postgresql mysql-client sqlite mongo-c-driver hiredis

# Install missing dependencies (Ubuntu/Debian)
sudo apt-get install libpq-dev libmysqlclient-dev libsqlite3-dev libmongoc-dev libhiredis-dev

# Check if libraries are found
pkg-config --libs libpq
mysql_config --libs
pkg-config --libs libmongoc-1.0
```

### Connection Issues

```bash
# Test PostgreSQL connection
psql -h localhost -U vexdb -d vexdb_test

# Test MySQL connection
mysql -h localhost -u vexdb -pvexdb_test vexdb_test

# Test MongoDB connection
mongosh "mongodb://vexdb:vexdb_test@localhost:27017/vexdb_test"

# Test Redis connection
redis-cli -h localhost ping
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: VexDB Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_PASSWORD: vexdb_test
          POSTGRES_USER: vexdb
          POSTGRES_DB: vexdb_test
        ports:
          - 5432:5432

      mysql:
        image: mysql:8.0
        env:
          MYSQL_ROOT_PASSWORD: vexdb_test
          MYSQL_DATABASE: vexdb_test
          MYSQL_USER: vexdb
          MYSQL_PASSWORD: vexdb_test
        ports:
          - 3306:3306

      mongodb:
        image: mongo:7
        env:
          MONGO_INITDB_ROOT_USERNAME: vexdb
          MONGO_INITDB_ROOT_PASSWORD: vexdb_test
        ports:
          - 27017:27017

      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v3

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libpq-dev libmysqlclient-dev libsqlite3-dev libmongoc-dev libhiredis-dev

      - name: Build and test
        run: |
          cd vex-clibs/vex_db
          make clean
          make HAVE_LIBPQ=1 HAVE_MYSQL=1 HAVE_MONGO=1 HAVE_REDIS=1 HAVE_SQLITE=1 \
            PQ_CFLAGS="$(pkg-config --cflags libpq)" \
            PQ_LDFLAGS="$(pkg-config --libs libpq)" \
            MYSQL_CFLAGS="$(pkg-config --cflags mysqlclient)" \
            MYSQL_LDFLAGS="$(pkg-config --libs mysqlclient)" \
            MONGO_CFLAGS="$(pkg-config --cflags libmongoc-1.0)" \
            MONGO_LDFLAGS="$(pkg-config --libs libmongoc-1.0)" \
            REDIS_LDFLAGS="-lhiredis" \
            test-all

      - name: Run integration tests
        run: |
          cd vex-clibs/vex_db
          ./test_integration
```

## Performance Testing

For performance testing and benchmarking, see the separate `BENCHMARKS.md` document.

## Contributing

When adding new features:

1. Add unit tests to `tests/test_all_drivers.c`
2. Add integration tests to `tests/test_integration.c`
3. Update this documentation
4. Ensure all tests pass: `./run_tests.sh`

## License

MIT
