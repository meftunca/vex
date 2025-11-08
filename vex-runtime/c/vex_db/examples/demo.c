
/* examples/demo.c — choose driver via argv: postgres|mysql|sqlite|mongo|redis */
#include "vex_db_driver.h"
#include <stdio.h>
#include <string.h>

#ifdef HAVE_LIBPQ
extern VexDbDriver VEX_DRIVER_POSTGRES;
#endif
#ifdef HAVE_MYSQL
extern VexDbDriver VEX_DRIVER_MYSQL;
#endif
#ifdef HAVE_SQLITE
extern VexDbDriver VEX_DRIVER_SQLITE;
#endif
#ifdef HAVE_MONGO
extern VexDbDriver VEX_DRIVER_MONGO;
#endif
#ifdef HAVE_REDIS
extern VexDbDriver VEX_DRIVER_REDIS;
#endif

static void run_demo(VexDbDriver *d, const char *conninfo, const char *query){
    printf("=== VexDB Demo: %s ===\n", d->driver_name);
    printf("Connecting to: %s\n", conninfo);
    
    VexConnection c = d->connect(conninfo);
    if (c.error.code != VEX_DB_OK){ 
        fprintf(stderr,"❌ Connect error: %s\n", c.error.message); 
        return; 
    }
    printf("✓ Connected successfully\n");
    
    printf("Executing query: %s\n", query);
    VexResultSet rs = d->execute_query(&c, query, 0, NULL);
    if (rs.error.code != VEX_DB_OK){ 
        fprintf(stderr,"❌ Exec error: %s\n", rs.error.message); 
        d->disconnect(&c); 
        return; 
    }
    printf("✓ Query executed\n\n");
    
    printf("Results:\n");
    printf("─────────────────────────────────────\n");
    int row_count = 0;
    for(;;){
        VexDbPayload p = d->fetch_next(&rs);
        if (!p.data && p.length==0) break; /* end */
        row_count++;
        printf("[%d] ", row_count);
        if (p.is_null) { 
            printf("(NULL)\n"); 
            continue; 
        }
        fwrite(p.data, 1, p.length, stdout); 
        fputc('\n', stdout);
    }
    printf("─────────────────────────────────────\n");
    printf("Total rows: %d\n", row_count);
    
    d->clear_result(&rs);
    d->disconnect(&c);
    printf("✓ Disconnected\n");
}

int main(int argc, char **argv){
    if (argc < 4){
        fprintf(stderr,"VexDB Multi-Driver Demo\n");
        fprintf(stderr,"usage: %s <driver> <conninfo> <query>\n\n", argv[0]);
        fprintf(stderr,"Supported drivers:\n");
#ifdef HAVE_LIBPQ
        fprintf(stderr,"  - postgres  (PostgreSQL via libpq)\n");
#endif
#ifdef HAVE_MYSQL
        fprintf(stderr,"  - mysql     (MySQL/MariaDB via libmysqlclient)\n");
#endif
#ifdef HAVE_SQLITE
        fprintf(stderr,"  - sqlite    (SQLite via sqlite3)\n");
#endif
#ifdef HAVE_MONGO
        fprintf(stderr,"  - mongo     (MongoDB via libmongoc)\n");
#endif
#ifdef HAVE_REDIS
        fprintf(stderr,"  - redis     (Redis via hiredis)\n");
#endif
        fprintf(stderr,"\nExamples:\n");
        fprintf(stderr,"  %s sqlite ./test.db \"SELECT 1\"\n", argv[0]);
        fprintf(stderr,"  %s postgres \"host=localhost user=postgres\" \"SELECT version()\"\n", argv[0]);
#ifdef HAVE_MONGO
        fprintf(stderr,"  %s mongo \"mongodb://localhost:27017/testdb\" \"users.find({})\"\n", argv[0]);
#endif
#ifdef HAVE_REDIS
        fprintf(stderr,"  %s redis \"localhost:6379\" \"GET mykey\"\n", argv[0]);
#endif
        return 1;
    }
    
    const char *drv = argv[1];
    
#ifdef HAVE_LIBPQ
    if (strcmp(drv,"postgres")==0 || strcmp(drv,"pg")==0) {
        run_demo(&VEX_DRIVER_POSTGRES, argv[2], argv[3]);
        return 0;
    }
#endif
#ifdef HAVE_MYSQL
    if (strcmp(drv,"mysql")==0) {
        run_demo(&VEX_DRIVER_MYSQL, argv[2], argv[3]);
        return 0;
    }
#endif
#ifdef HAVE_SQLITE
    if (strcmp(drv,"sqlite")==0) {
        run_demo(&VEX_DRIVER_SQLITE, argv[2], argv[3]);
        return 0;
    }
#endif
#ifdef HAVE_MONGO
    if (strcmp(drv,"mongo")==0 || strcmp(drv,"mongodb")==0) {
        run_demo(&VEX_DRIVER_MONGO, argv[2], argv[3]);
        return 0;
    }
#endif
#ifdef HAVE_REDIS
    if (strcmp(drv,"redis")==0) {
        run_demo(&VEX_DRIVER_REDIS, argv[2], argv[3]);
        return 0;
    }
#endif
    
    fprintf(stderr,"❌ Unknown or disabled driver: %s\n", drv);
    fprintf(stderr,"Recompile with appropriate HAVE_* flags to enable drivers.\n");
    return 1;
}
