#!/bin/bash

# VexDB Test Runner Script
# This script starts Docker containers and runs all tests

set -e

echo "═══════════════════════════════════════════════════════"
echo "  VexDB Test Runner"
echo "═══════════════════════════════════════════════════════"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running. Please start Docker and try again.${NC}"
    exit 1
fi

echo -e "\n${YELLOW}▶ Starting Docker Compose...${NC}"
docker-compose up -d

echo -e "\n${YELLOW}▶ Waiting for databases to be ready...${NC}"
sleep 5

# Wait for PostgreSQL
echo -n "  Waiting for PostgreSQL... "
for i in {1..30}; do
    if docker-compose exec -T postgres pg_isready -U vexdb > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo -e "${RED}✗ (timeout)${NC}"
    fi
done

# Wait for MySQL
echo -n "  Waiting for MySQL... "
for i in {1..30}; do
    if docker-compose exec -T mysql mysqladmin ping -h localhost -u root -pvexdb_test > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo -e "${RED}✗ (timeout)${NC}"
    fi
done

# Wait for MongoDB
echo -n "  Waiting for MongoDB... "
for i in {1..30}; do
    if docker-compose exec -T mongodb mongosh --quiet --eval "db.adminCommand('ping')" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo -e "${RED}✗ (timeout)${NC}"
    fi
done

# Wait for Redis
echo -n "  Waiting for Redis... "
for i in {1..30}; do
    if docker-compose exec -T redis redis-cli ping > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo -e "${RED}✗ (timeout)${NC}"
    fi
done

echo -e "\n${YELLOW}▶ Building tests...${NC}"

# Set up PKG_CONFIG_PATH for keg-only libraries on macOS
if [ -d "/opt/homebrew" ]; then
    export PKG_CONFIG_PATH="/opt/homebrew/opt/libpq/lib/pkgconfig:/opt/homebrew/opt/mysql-client/lib/pkgconfig:/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH"
fi

# Detect library paths
if [ -d "/opt/homebrew/opt/libpq" ]; then
    PQ_CFLAGS="-I/opt/homebrew/opt/libpq/include"
    PQ_LDFLAGS="-L/opt/homebrew/opt/libpq/lib -lpq"
else
    PQ_CFLAGS="$(pkg-config --cflags libpq 2>/dev/null || echo '')"
    PQ_LDFLAGS="$(pkg-config --libs libpq 2>/dev/null || echo '-lpq')"
fi

if [ -d "/opt/homebrew/opt/mysql-client" ]; then
    MYSQL_CFLAGS="-I/opt/homebrew/opt/mysql-client/include/mysql"
    MYSQL_LDFLAGS="-L/opt/homebrew/opt/mysql-client/lib -lmysqlclient"
else
    MYSQL_CFLAGS="$(mysql_config --cflags 2>/dev/null || pkg-config --cflags mysqlclient 2>/dev/null || echo '')"
    MYSQL_LDFLAGS="$(mysql_config --libs 2>/dev/null || pkg-config --libs mysqlclient 2>/dev/null || echo '-lmysqlclient')"
fi

MONGO_CFLAGS="$(pkg-config --cflags libmongoc-1.0 2>/dev/null || echo '')"
MONGO_LDFLAGS="$(pkg-config --libs libmongoc-1.0 2>/dev/null || echo '-lmongoc-1.0 -lbson-1.0')"

make clean
# Build without MongoDB for now (requires additional setup)
make HAVE_LIBPQ=1 HAVE_MYSQL=1 HAVE_REDIS=1 HAVE_SQLITE=1 \
  PQ_CFLAGS="$PQ_CFLAGS" \
  PQ_LDFLAGS="$PQ_LDFLAGS" \
  MYSQL_CFLAGS="$MYSQL_CFLAGS" \
  MYSQL_LDFLAGS="$MYSQL_LDFLAGS" \
  REDIS_LDFLAGS="-lhiredis" \
  test_integration

echo -e "\n${YELLOW}▶ Running unit tests...${NC}"
make test || true

echo -e "\n${YELLOW}▶ Running integration tests...${NC}"
./test_integration

TEST_EXIT_CODE=$?

echo -e "\n${YELLOW}▶ Cleaning up...${NC}"
# Keep containers running for manual inspection if tests failed
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed, stopping containers...${NC}"
    docker-compose down
else
    echo -e "${RED}✗ Some tests failed. Containers are still running for inspection.${NC}"
    echo -e "${YELLOW}  To stop containers: docker-compose down${NC}"
    echo -e "${YELLOW}  To view logs: docker-compose logs <service>${NC}"
fi

exit $TEST_EXIT_CODE

