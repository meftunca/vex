
/* tests/test_all_drivers.c — Comprehensive test suite for all VexDB drivers */
#include "vex_db_driver.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define TEST_PASSED "✅"
#define TEST_FAILED "❌"
#define TEST_SKIPPED "⊘"

static int tests_run = 0;
static int tests_passed = 0;
static int tests_failed = 0;
static int tests_skipped = 0;

#define TEST(name) \
    printf("\n  Testing: %s ... ", name); \
    tests_run++;

#define PASS() \
    do { \
        printf("%s PASS\n", TEST_PASSED); \
        tests_passed++; \
    } while(0)

#define FAIL(msg) \
    do { \
        printf("%s FAIL: %s\n", TEST_FAILED, msg); \
        tests_failed++; \
    } while(0)

#define SKIP(msg) \
    do { \
        printf("%s SKIP: %s\n", TEST_SKIPPED, msg); \
        tests_skipped++; \
    } while(0)

#define ASSERT(cond, msg) \
    if (!(cond)) { \
        FAIL(msg); \
        return; \
    }

/* SQLite Tests */
#ifdef HAVE_SQLITE
static void test_sqlite_basic() {
    TEST("SQLite basic connection and query");
    
    extern VexDbDriver VEX_DRIVER_SQLITE;
    
    // Create in-memory database
    VexConnection conn = VEX_DRIVER_SQLITE.connect(":memory:");
    ASSERT(conn.error.code == VEX_DB_OK, conn.error.message);
    
    // Create table
    VexResultSet rs = VEX_DRIVER_SQLITE.execute_query(&conn, 
        "CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    VEX_DRIVER_SQLITE.clear_result(&rs);
    
    // Insert data
    rs = VEX_DRIVER_SQLITE.execute_query(&conn,
        "INSERT INTO test VALUES (1, 'Alice'), (2, 'Bob')", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    VEX_DRIVER_SQLITE.clear_result(&rs);
    
    // Query data
    rs = VEX_DRIVER_SQLITE.execute_query(&conn, "SELECT name FROM test ORDER BY id", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    
    VexDbPayload p1 = VEX_DRIVER_SQLITE.fetch_next(&rs);
    ASSERT(p1.data && strncmp(p1.data, "Alice", 5) == 0, "Expected 'Alice'");
    
    VexDbPayload p2 = VEX_DRIVER_SQLITE.fetch_next(&rs);
    ASSERT(p2.data && strncmp(p2.data, "Bob", 3) == 0, "Expected 'Bob'");
    
    VexDbPayload p3 = VEX_DRIVER_SQLITE.fetch_next(&rs);
    ASSERT(!p3.data && p3.length == 0, "Expected end of results");
    
    VEX_DRIVER_SQLITE.clear_result(&rs);
    VEX_DRIVER_SQLITE.disconnect(&conn);
    
    PASS();
}

static void test_sqlite_null_handling() {
    TEST("SQLite NULL handling");
    
    extern VexDbDriver VEX_DRIVER_SQLITE;
    VexConnection conn = VEX_DRIVER_SQLITE.connect(":memory:");
    ASSERT(conn.error.code == VEX_DB_OK, conn.error.message);
    
    VexResultSet rs = VEX_DRIVER_SQLITE.execute_query(&conn,
        "SELECT NULL", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    
    VexDbPayload p = VEX_DRIVER_SQLITE.fetch_next(&rs);
    ASSERT(p.is_null, "Expected NULL value");
    
    VEX_DRIVER_SQLITE.clear_result(&rs);
    VEX_DRIVER_SQLITE.disconnect(&conn);
    
    PASS();
}
#endif

/* PostgreSQL Tests */
#ifdef HAVE_LIBPQ
static void test_postgres_basic() {
    TEST("PostgreSQL basic connection and query");
    
    extern VexDbDriver VEX_DRIVER_POSTGRES;
    
    // Try to connect to local PostgreSQL
    VexConnection conn = VEX_DRIVER_POSTGRES.connect("host=localhost");
    
    if (conn.error.code != VEX_DB_OK) {
        SKIP("PostgreSQL not available");
        return;
    }
    
    VexResultSet rs = VEX_DRIVER_POSTGRES.execute_query(&conn, "SELECT 42", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    
    VexDbPayload p = VEX_DRIVER_POSTGRES.fetch_next(&rs);
    ASSERT(p.data && strncmp(p.data, "42", 2) == 0, "Expected '42'");
    
    VEX_DRIVER_POSTGRES.clear_result(&rs);
    VEX_DRIVER_POSTGRES.disconnect(&conn);
    
    PASS();
}
#endif

/* MySQL Tests */
#ifdef HAVE_MYSQL
static void test_mysql_basic() {
    TEST("MySQL basic connection and query");
    
    extern VexDbDriver VEX_DRIVER_MYSQL;
    
    // Try to connect to local MySQL
    VexConnection conn = VEX_DRIVER_MYSQL.connect("host=localhost");
    
    if (conn.error.code != VEX_DB_OK) {
        SKIP("MySQL not available");
        return;
    }
    
    VexResultSet rs = VEX_DRIVER_MYSQL.execute_query(&conn, "SELECT 42", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    
    VexDbPayload p = VEX_DRIVER_MYSQL.fetch_next(&rs);
    ASSERT(p.data && strncmp(p.data, "42", 2) == 0, "Expected '42'");
    
    VEX_DRIVER_MYSQL.clear_result(&rs);
    VEX_DRIVER_MYSQL.disconnect(&conn);
    
    PASS();
}
#endif

/* MongoDB Tests */
#ifdef HAVE_MONGO
static void test_mongo_basic() {
    TEST("MongoDB basic connection and query");
    
    extern VexDbDriver VEX_DRIVER_MONGO;
    
    // Try to connect to local MongoDB
    VexConnection conn = VEX_DRIVER_MONGO.connect("mongodb://localhost:27017/vexdb_test");
    
    if (conn.error.code != VEX_DB_OK) {
        SKIP("MongoDB not available");
        return;
    }
    
    // Try to find documents (empty result is OK)
    VexResultSet rs = VEX_DRIVER_MONGO.execute_query(&conn, "test_collection.find({})", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    
    // Just verify we can fetch (might be empty)
    VexDbPayload p = VEX_DRIVER_MONGO.fetch_next(&rs);
    // Empty result is OK for test
    
    VEX_DRIVER_MONGO.clear_result(&rs);
    VEX_DRIVER_MONGO.disconnect(&conn);
    
    PASS();
}

static void test_mongo_json_output() {
    TEST("MongoDB JSON document output");
    
    extern VexDbDriver VEX_DRIVER_MONGO;
    
    VexConnection conn = VEX_DRIVER_MONGO.connect("mongodb://localhost:27017/vexdb_test");
    
    if (conn.error.code != VEX_DB_OK) {
        SKIP("MongoDB not available");
        return;
    }
    
    // Query should return JSON even if empty
    VexResultSet rs = VEX_DRIVER_MONGO.execute_query(&conn, "test_collection.find({})", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    ASSERT(rs.column_count == 1, "Expected 1 column for JSON documents");
    
    VEX_DRIVER_MONGO.clear_result(&rs);
    VEX_DRIVER_MONGO.disconnect(&conn);
    
    PASS();
}
#endif

/* Redis Tests */
#ifdef HAVE_REDIS
static void test_redis_basic() {
    TEST("Redis basic connection and PING");
    
    extern VexDbDriver VEX_DRIVER_REDIS;
    
    // Try to connect to local Redis
    VexConnection conn = VEX_DRIVER_REDIS.connect("localhost:6379");
    
    if (conn.error.code != VEX_DB_OK) {
        SKIP("Redis not available");
        return;
    }
    
    VexResultSet rs = VEX_DRIVER_REDIS.execute_query(&conn, "PING", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    
    VexDbPayload p = VEX_DRIVER_REDIS.fetch_next(&rs);
    ASSERT(p.data, "Expected PONG response");
    
    VEX_DRIVER_REDIS.clear_result(&rs);
    VEX_DRIVER_REDIS.disconnect(&conn);
    
    PASS();
}

static void test_redis_set_get() {
    TEST("Redis SET/GET operations");
    
    extern VexDbDriver VEX_DRIVER_REDIS;
    
    VexConnection conn = VEX_DRIVER_REDIS.connect("localhost:6379");
    
    if (conn.error.code != VEX_DB_OK) {
        SKIP("Redis not available");
        return;
    }
    
    // SET a test key
    VexResultSet rs = VEX_DRIVER_REDIS.execute_query(&conn, "SET vexdb_test_key test_value", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    // GET the key
    rs = VEX_DRIVER_REDIS.execute_query(&conn, "GET vexdb_test_key", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    
    VexDbPayload p = VEX_DRIVER_REDIS.fetch_next(&rs);
    ASSERT(p.data && strncmp(p.data, "test_value", 10) == 0, "Expected 'test_value'");
    
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    // Cleanup
    rs = VEX_DRIVER_REDIS.execute_query(&conn, "DEL vexdb_test_key", 0, NULL);
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    VEX_DRIVER_REDIS.disconnect(&conn);
    
    PASS();
}

static void test_redis_list_operations() {
    TEST("Redis list operations");
    
    extern VexDbDriver VEX_DRIVER_REDIS;
    
    VexConnection conn = VEX_DRIVER_REDIS.connect("localhost:6379");
    
    if (conn.error.code != VEX_DB_OK) {
        SKIP("Redis not available");
        return;
    }
    
    // Create a list
    VexResultSet rs = VEX_DRIVER_REDIS.execute_query(&conn, "RPUSH vexdb_test_list item1 item2 item3", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    // Get list range
    rs = VEX_DRIVER_REDIS.execute_query(&conn, "LRANGE vexdb_test_list 0 -1", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    
    int count = 0;
    for (;;) {
        VexDbPayload p = VEX_DRIVER_REDIS.fetch_next(&rs);
        if (!p.data && p.length == 0) break;
        count++;
    }
    ASSERT(count == 3, "Expected 3 items in list");
    
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    // Cleanup
    rs = VEX_DRIVER_REDIS.execute_query(&conn, "DEL vexdb_test_list", 0, NULL);
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    VEX_DRIVER_REDIS.disconnect(&conn);
    
    PASS();
}
#endif

/* Error Handling Tests */
static void test_error_handling() {
    TEST("Error handling for invalid connections");
    
#ifdef HAVE_SQLITE
    extern VexDbDriver VEX_DRIVER_SQLITE;
    
    // Try invalid file path
    VexConnection conn = VEX_DRIVER_SQLITE.connect("/invalid/path/to/db.sqlite");
    ASSERT(conn.error.code != VEX_DB_OK, "Expected connection error");
    if (conn.native_conn) VEX_DRIVER_SQLITE.disconnect(&conn);
#endif
    
    PASS();
}

static void test_invalid_query() {
    TEST("Error handling for invalid queries");
    
#ifdef HAVE_SQLITE
    extern VexDbDriver VEX_DRIVER_SQLITE;
    
    VexConnection conn = VEX_DRIVER_SQLITE.connect(":memory:");
    ASSERT(conn.error.code == VEX_DB_OK, conn.error.message);
    
    VexResultSet rs = VEX_DRIVER_SQLITE.execute_query(&conn, "INVALID SQL SYNTAX", 0, NULL);
    ASSERT(rs.error.code != VEX_DB_OK, "Expected query error");
    
    VEX_DRIVER_SQLITE.clear_result(&rs);
    VEX_DRIVER_SQLITE.disconnect(&conn);
#endif
    
    PASS();
}

/* Driver Capabilities Tests */
static void test_driver_capabilities() {
    TEST("Driver capabilities reporting");
    
#ifdef HAVE_LIBPQ
    extern VexDbDriver VEX_DRIVER_POSTGRES;
    ASSERT(VEX_DRIVER_POSTGRES.capabilities & VEX_CAP_SQL, "PostgreSQL should support SQL");
    ASSERT(VEX_DRIVER_POSTGRES.capabilities & VEX_CAP_ASYNC, "PostgreSQL should support async");
#endif

#ifdef HAVE_SQLITE
    extern VexDbDriver VEX_DRIVER_SQLITE;
    ASSERT(VEX_DRIVER_SQLITE.capabilities & VEX_CAP_SQL, "SQLite should support SQL");
#endif
    
    PASS();
}

/* Test Runner */
int main() {
    printf("═══════════════════════════════════════════════════════\n");
    printf("  VexDB Comprehensive Test Suite\n");
    printf("═══════════════════════════════════════════════════════\n");
    
    printf("\n▶ SQLite Tests:\n");
#ifdef HAVE_SQLITE
    test_sqlite_basic();
    test_sqlite_null_handling();
#else
    printf("  SQLite support not compiled\n");
#endif
    
    printf("\n▶ PostgreSQL Tests:\n");
#ifdef HAVE_LIBPQ
    test_postgres_basic();
#else
    printf("  PostgreSQL support not compiled\n");
#endif
    
    printf("\n▶ MySQL Tests:\n");
#ifdef HAVE_MYSQL
    test_mysql_basic();
#else
    printf("  MySQL support not compiled\n");
#endif
    
    printf("\n▶ MongoDB Tests:\n");
#ifdef HAVE_MONGO
    test_mongo_basic();
    test_mongo_json_output();
#else
    printf("  MongoDB support not compiled\n");
#endif
    
    printf("\n▶ Redis Tests:\n");
#ifdef HAVE_REDIS
    test_redis_basic();
    test_redis_set_get();
    test_redis_list_operations();
#else
    printf("  Redis support not compiled\n");
#endif
    
    printf("\n▶ Error Handling Tests:\n");
    test_error_handling();
    test_invalid_query();
    
    printf("\n▶ General Tests:\n");
    test_driver_capabilities();
    
    printf("\n═══════════════════════════════════════════════════════\n");
    printf("  Test Results:\n");
    printf("═══════════════════════════════════════════════════════\n");
    printf("  Total:   %d\n", tests_run);
    printf("  %s Passed: %d\n", TEST_PASSED, tests_passed);
    printf("  %s Failed: %d\n", TEST_FAILED, tests_failed);
    printf("  %s Skipped: %d\n", TEST_SKIPPED, tests_skipped);
    printf("═══════════════════════════════════════════════════════\n");
    
    if (tests_failed > 0) {
        printf("\n❌ SOME TESTS FAILED\n\n");
        return 1;
    } else {
        printf("\n✅ ALL TESTS PASSED\n\n");
        return 0;
    }
}

