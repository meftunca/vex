/* tests/test_integration.c — Real integration tests with Docker databases */
#include "vex_db_driver.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/select.h>
#include <pthread.h>

#define TEST_PASSED "✅"
#define TEST_FAILED "❌"
#define TEST_SKIPPED "⊘"

static int tests_run = 0;
static int tests_passed = 0;
static int tests_failed = 0;
static int tests_skipped = 0;

#define TEST(name) \
    printf("\n  Testing: %s ... ", name); \
    fflush(stdout); \
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

/* ========== PostgreSQL Tests ========== */
#ifdef HAVE_LIBPQ

static void test_pg_full_crud() {
    TEST("PostgreSQL - Full CRUD operations");
    
    extern VexDbDriver VEX_DRIVER_POSTGRES;
    
    VexConnection conn = VEX_DRIVER_POSTGRES.connect("host=localhost user=vexdb password=vexdb_test dbname=vexdb_test");
    if (conn.error.code != VEX_DB_OK) {
        SKIP("PostgreSQL not available");
        return;
    }
    
    // CREATE
    VexResultSet rs = VEX_DRIVER_POSTGRES.execute_query(&conn, 
        "CREATE TABLE IF NOT EXISTS test_users (id SERIAL PRIMARY KEY, name TEXT, age INT)", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    VEX_DRIVER_POSTGRES.clear_result(&rs);
    
    // Clean previous data
    rs = VEX_DRIVER_POSTGRES.execute_query(&conn, "TRUNCATE test_users RESTART IDENTITY", 0, NULL);
    VEX_DRIVER_POSTGRES.clear_result(&rs);
    
    // INSERT
    rs = VEX_DRIVER_POSTGRES.execute_query(&conn, 
        "INSERT INTO test_users (name, age) VALUES ('Alice', 30), ('Bob', 25)", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    VEX_DRIVER_POSTGRES.clear_result(&rs);
    
    // SELECT
    rs = VEX_DRIVER_POSTGRES.execute_query(&conn, 
        "SELECT name, age FROM test_users ORDER BY id", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    
    int count = 0;
    for (;;) {
        VexDbPayload p = VEX_DRIVER_POSTGRES.fetch_next(&rs);
        if (!p.data && p.length == 0) break;
        count++;
    }
    ASSERT(count == 2, "Expected 2 rows");
    VEX_DRIVER_POSTGRES.clear_result(&rs);
    
    // UPDATE
    rs = VEX_DRIVER_POSTGRES.execute_query(&conn, 
        "UPDATE test_users SET age = 31 WHERE name = 'Alice'", 0, NULL);
    VEX_DRIVER_POSTGRES.clear_result(&rs);
    
    // DELETE
    rs = VEX_DRIVER_POSTGRES.execute_query(&conn, 
        "DELETE FROM test_users WHERE name = 'Bob'", 0, NULL);
    VEX_DRIVER_POSTGRES.clear_result(&rs);
    
    VEX_DRIVER_POSTGRES.disconnect(&conn);
    PASS();
}

static void test_pg_transactions() {
    TEST("PostgreSQL - Transaction support");
    
    extern VexDbDriver VEX_DRIVER_POSTGRES;
    
    VexConnection conn = VEX_DRIVER_POSTGRES.connect("host=localhost user=vexdb password=vexdb_test dbname=vexdb_test");
    if (conn.error.code != VEX_DB_OK) {
        SKIP("PostgreSQL not available");
        return;
    }
    
    // Begin transaction
    int result = VEX_DRIVER_POSTGRES.begin_transaction(&conn);
    ASSERT(result == 0, "Failed to begin transaction");
    
    // Insert in transaction
    VexResultSet rs = VEX_DRIVER_POSTGRES.execute_query(&conn, 
        "INSERT INTO test_users (name, age) VALUES ('Charlie', 35)", 0, NULL);
    VEX_DRIVER_POSTGRES.clear_result(&rs);
    
    // Rollback
    result = VEX_DRIVER_POSTGRES.rollback_transaction(&conn);
    ASSERT(result == 0, "Failed to rollback");
    
    // Verify rollback
    rs = VEX_DRIVER_POSTGRES.execute_query(&conn, 
        "SELECT COUNT(*) FROM test_users WHERE name = 'Charlie'", 0, NULL);
    VexDbPayload p = VEX_DRIVER_POSTGRES.fetch_next(&rs);
    ASSERT(p.data && p.data[0] == '0', "Transaction not rolled back");
    VEX_DRIVER_POSTGRES.clear_result(&rs);
    
    // Test commit
    VEX_DRIVER_POSTGRES.begin_transaction(&conn);
    rs = VEX_DRIVER_POSTGRES.execute_query(&conn, 
        "INSERT INTO test_users (name, age) VALUES ('Diana', 28)", 0, NULL);
    VEX_DRIVER_POSTGRES.clear_result(&rs);
    result = VEX_DRIVER_POSTGRES.commit_transaction(&conn);
    ASSERT(result == 0, "Failed to commit");
    
    VEX_DRIVER_POSTGRES.disconnect(&conn);
    PASS();
}

static void test_pg_listen_notify() {
    TEST("PostgreSQL - LISTEN/NOTIFY");
    
    extern VexDbDriver VEX_DRIVER_POSTGRES;
    
    VexConnection conn1 = VEX_DRIVER_POSTGRES.connect("host=localhost user=vexdb password=vexdb_test dbname=vexdb_test");
    VexConnection conn2 = VEX_DRIVER_POSTGRES.connect("host=localhost user=vexdb password=vexdb_test dbname=vexdb_test");
    
    if (conn1.error.code != VEX_DB_OK || conn2.error.code != VEX_DB_OK) {
        SKIP("PostgreSQL not available");
        return;
    }
    
    // Subscribe on conn1
    int result = VEX_DRIVER_POSTGRES.subscribe(&conn1, "test_channel");
    ASSERT(result == 0, "Failed to subscribe");
    
    // Publish on conn2
    result = VEX_DRIVER_POSTGRES.publish(&conn2, "test_channel", "Hello VexDB!", 12);
    ASSERT(result == 0, "Failed to publish");
    
    // Poll for notification on conn1
    sleep(1); // Give time for notification
    result = VEX_DRIVER_POSTGRES.poll_notifications(&conn1);
    ASSERT(result > 0, "No notification received");
    
    // Get notification
    VexDbPayload notif = VEX_DRIVER_POSTGRES.get_notification(&conn1);
    ASSERT(notif.data != NULL, "Notification data is NULL");
    ASSERT(strstr(notif.data, "test_channel") != NULL, "Channel name not in notification");
    ASSERT(strstr(notif.data, "Hello VexDB!") != NULL, "Message not in notification");
    
    VEX_DRIVER_POSTGRES.disconnect(&conn1);
    VEX_DRIVER_POSTGRES.disconnect(&conn2);
    PASS();
}

static void test_pg_cursor_streaming() {
    TEST("PostgreSQL - Cursor-based streaming");
    
    extern VexDbDriver VEX_DRIVER_POSTGRES;
    
    VexConnection conn = VEX_DRIVER_POSTGRES.connect("host=localhost user=vexdb password=vexdb_test dbname=vexdb_test");
    if (conn.error.code != VEX_DB_OK) {
        SKIP("PostgreSQL not available");
        return;
    }
    
    // Insert test data
    VEX_DRIVER_POSTGRES.begin_transaction(&conn);
    for (int i = 0; i < 100; i++) {
        char query[256];
        snprintf(query, sizeof(query), "INSERT INTO test_users (name, age) VALUES ('User%d', %d)", i, 20 + i % 50);
        VexResultSet rs = VEX_DRIVER_POSTGRES.execute_query(&conn, query, 0, NULL);
        VEX_DRIVER_POSTGRES.clear_result(&rs);
    }
    VEX_DRIVER_POSTGRES.commit_transaction(&conn);
    
    // Start transaction for cursor
    VEX_DRIVER_POSTGRES.begin_transaction(&conn);
    
    // Declare cursor
    int result = VEX_DRIVER_POSTGRES.declare_cursor(&conn, "test_cursor", 
        "SELECT name, age FROM test_users WHERE name LIKE 'User%' ORDER BY id");
    ASSERT(result == 0, "Failed to declare cursor");
    
    // Fetch in batches
    int total_rows = 0;
    for (int batch = 0; batch < 5; batch++) {
        VexResultSet rs = VEX_DRIVER_POSTGRES.fetch_from_cursor(&conn, "test_cursor", 20);
        ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
        
        for (;;) {
            VexDbPayload p = VEX_DRIVER_POSTGRES.fetch_next(&rs);
            if (!p.data && p.length == 0) break;
            total_rows++;
        }
        VEX_DRIVER_POSTGRES.clear_result(&rs);
    }
    
    ASSERT(total_rows == 100, "Expected 100 rows from cursor");
    
    // Close cursor
    result = VEX_DRIVER_POSTGRES.close_cursor(&conn, "test_cursor");
    ASSERT(result == 0, "Failed to close cursor");
    
    VEX_DRIVER_POSTGRES.commit_transaction(&conn);
    VEX_DRIVER_POSTGRES.disconnect(&conn);
    PASS();
}

#endif

/* ========== MySQL Tests ========== */
#ifdef HAVE_MYSQL

static void test_mysql_full_crud() {
    TEST("MySQL - Full CRUD operations");
    
    extern VexDbDriver VEX_DRIVER_MYSQL;
    
    VexConnection conn = VEX_DRIVER_MYSQL.connect("host=localhost user=vexdb password=vexdb_test db=vexdb_test port=3306");
    if (conn.error.code != VEX_DB_OK) {
        SKIP("MySQL not available");
        return;
    }
    
    // CREATE
    VexResultSet rs = VEX_DRIVER_MYSQL.execute_query(&conn, 
        "CREATE TABLE IF NOT EXISTS test_products (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100), price DECIMAL(10,2))", 0, NULL);
    VEX_DRIVER_MYSQL.clear_result(&rs);
    
    // TRUNCATE
    rs = VEX_DRIVER_MYSQL.execute_query(&conn, "TRUNCATE test_products", 0, NULL);
    VEX_DRIVER_MYSQL.clear_result(&rs);
    
    // INSERT
    rs = VEX_DRIVER_MYSQL.execute_query(&conn, 
        "INSERT INTO test_products (name, price) VALUES ('Product A', 19.99), ('Product B', 29.99)", 0, NULL);
    VEX_DRIVER_MYSQL.clear_result(&rs);
    
    // SELECT
    rs = VEX_DRIVER_MYSQL.execute_query(&conn, "SELECT name FROM test_products ORDER BY id", 0, NULL);
    int count = 0;
    for (;;) {
        VexDbPayload p = VEX_DRIVER_MYSQL.fetch_next(&rs);
        if (!p.data && p.length == 0) break;
        count++;
    }
    ASSERT(count == 2, "Expected 2 products");
    VEX_DRIVER_MYSQL.clear_result(&rs);
    
    VEX_DRIVER_MYSQL.disconnect(&conn);
    PASS();
}

static void test_mysql_transactions() {
    TEST("MySQL - Transaction support");
    
    extern VexDbDriver VEX_DRIVER_MYSQL;
    
    VexConnection conn = VEX_DRIVER_MYSQL.connect("host=localhost user=vexdb password=vexdb_test db=vexdb_test port=3306");
    if (conn.error.code != VEX_DB_OK) {
        SKIP("MySQL not available");
        return;
    }
    
    VEX_DRIVER_MYSQL.begin_transaction(&conn);
    VexResultSet rs = VEX_DRIVER_MYSQL.execute_query(&conn, 
        "INSERT INTO test_products (name, price) VALUES ('Product C', 39.99)", 0, NULL);
    VEX_DRIVER_MYSQL.clear_result(&rs);
    VEX_DRIVER_MYSQL.rollback_transaction(&conn);
    
    VEX_DRIVER_MYSQL.disconnect(&conn);
    PASS();
}

#endif

/* ========== Redis Tests ========== */
#ifdef HAVE_REDIS

static void test_redis_operations() {
    TEST("Redis - Basic operations");
    
    extern VexDbDriver VEX_DRIVER_REDIS;
    
    VexConnection conn = VEX_DRIVER_REDIS.connect("localhost:6379");
    if (conn.error.code != VEX_DB_OK) {
        SKIP("Redis not available");
        return;
    }
    
    // SET
    VexResultSet rs = VEX_DRIVER_REDIS.execute_query(&conn, "SET test_key test_value", 0, NULL);
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    // GET
    rs = VEX_DRIVER_REDIS.execute_query(&conn, "GET test_key", 0, NULL);
    VexDbPayload p = VEX_DRIVER_REDIS.fetch_next(&rs);
    ASSERT(p.data && strncmp(p.data, "test_value", 10) == 0, "Value mismatch");
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    // DEL
    rs = VEX_DRIVER_REDIS.execute_query(&conn, "DEL test_key", 0, NULL);
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    VEX_DRIVER_REDIS.disconnect(&conn);
    PASS();
}

static void test_redis_transactions() {
    TEST("Redis - Transaction support (MULTI/EXEC)");
    
    extern VexDbDriver VEX_DRIVER_REDIS;
    
    VexConnection conn = VEX_DRIVER_REDIS.connect("localhost:6379");
    if (conn.error.code != VEX_DB_OK) {
        SKIP("Redis not available");
        return;
    }
    
    int result = VEX_DRIVER_REDIS.begin_transaction(&conn);
    ASSERT(result == 0, "Failed to begin transaction");
    
    VexResultSet rs = VEX_DRIVER_REDIS.execute_query(&conn, "SET txn_key1 value1", 0, NULL);
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    rs = VEX_DRIVER_REDIS.execute_query(&conn, "SET txn_key2 value2", 0, NULL);
    VEX_DRIVER_REDIS.clear_result(&rs);
    
    result = VEX_DRIVER_REDIS.commit_transaction(&conn);
    ASSERT(result == 0, "Failed to commit transaction");
    
    VEX_DRIVER_REDIS.disconnect(&conn);
    PASS();
}

#endif

/* ========== MongoDB Tests ========== */
#ifdef HAVE_MONGO

static void test_mongodb_operations() {
    TEST("MongoDB - Basic operations");
    
    extern VexDbDriver VEX_DRIVER_MONGO;
    
    VexConnection conn = VEX_DRIVER_MONGO.connect("mongodb://vexdb:vexdb_test@localhost:27017/vexdb_test");
    if (conn.error.code != VEX_DB_OK) {
        SKIP("MongoDB not available");
        return;
    }
    
    // Find (may be empty)
    VexResultSet rs = VEX_DRIVER_MONGO.execute_query(&conn, "test_collection.find({})", 0, NULL);
    ASSERT(rs.error.code == VEX_DB_OK, rs.error.message);
    
    int count = 0;
    for (;;) {
        VexDbPayload p = VEX_DRIVER_MONGO.fetch_next(&rs);
        if (!p.data && p.length == 0) break;
        count++;
    }
    
    VEX_DRIVER_MONGO.clear_result(&rs);
    VEX_DRIVER_MONGO.disconnect(&conn);
    PASS();
}

#endif

/* ========== Main Test Runner ========== */

int main() {
    printf("═══════════════════════════════════════════════════════\n");
    printf("  VexDB Integration Test Suite (Docker)\n");
    printf("═══════════════════════════════════════════════════════\n");
    
    printf("\n▶ PostgreSQL Tests:\n");
#ifdef HAVE_LIBPQ
    test_pg_full_crud();
    test_pg_transactions();
    test_pg_listen_notify();
    test_pg_cursor_streaming();
#else
    printf("  PostgreSQL support not compiled\n");
#endif
    
    printf("\n▶ MySQL Tests:\n");
#ifdef HAVE_MYSQL
    test_mysql_full_crud();
    test_mysql_transactions();
#else
    printf("  MySQL support not compiled\n");
#endif
    
    printf("\n▶ Redis Tests:\n");
#ifdef HAVE_REDIS
    test_redis_operations();
    test_redis_transactions();
#else
    printf("  Redis support not compiled\n");
#endif
    
    printf("\n▶ MongoDB Tests:\n");
#ifdef HAVE_MONGO
    test_mongodb_operations();
#else
    printf("  MongoDB support not compiled\n");
#endif
    
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

