/* vex_db_demo.c — minimal demonstration
 * Shows how to use the driver synchronously; comments show how to adapt this
 * into Vex channels.
 */
#include "vex_db_driver.h"
#include <stdio.h>
#include <string.h>

extern VexDbDriver VEX_DRIVER_POSTGRES;
extern VexDbDriver VEX_DRIVER_MONGODB;

static void run_sql_demo(const char *conninfo, const char *query){
    VexDbDriver *d = &VEX_DRIVER_POSTGRES;
    VexConnection c = d->connect(conninfo);
    if (c.error.code != VEX_DB_OK) {
        fprintf(stderr, "connect error: %s\n", c.error.message);
        return;
    }
    VexResultSet rs = d->execute_query(&c, query, 0, NULL);
    if (rs.error.code != VEX_DB_OK) {
        fprintf(stderr, "exec error: %s\n", rs.error.message);
        d->disconnect(&c);
        return;
    }
    for (;;) {
        VexDbPayload p = d->fetch_next(&rs);
        if (!p.data) break;
        fwrite(p.data, 1, p.length, stdout);
        fputc('\n', stdout);
    }
    d->clear_result(&rs);
    d->disconnect(&c);
}

static void run_doc_demo(const char *conninfo){
    VexDbDriver *d = &VEX_DRIVER_MONGODB;
    VexConnection c = d->connect(conninfo);
    VexDbPayload filter = {0}; /* mock ignores filter */
    VexResultSet rs = d->find_doc(&c, "users", filter);
    for (;;) {
        VexDbPayload p = d->fetch_next(&rs);
        if (!p.data) break;
        fwrite(p.data, 1, p.length, stdout);
        fputc('\n', stdout);
    }
    d->clear_result(&rs);
    d->disconnect(&c);
}

int main(int argc, char **argv){
    if (argc >= 3) {
        run_sql_demo(argv[1], argv[2]);
    } else {
        /* mock demos */
        printf("=== Mock Postgres ===\n");
        run_sql_demo("host=mock", "select 1");
        printf("=== Mock Mongo ===\n");
        run_doc_demo("mongodb://localhost");
    }
    return 0;
}

/* -------------------- VEX CHANNEL ADAPTER (commentary) --------------------
In Vex, you would wrap the pull iterator into channels like:

type Row { payload: *VexDbPayload }
type DbRequest { query: string, params: []Param, rows: chan Row, done: chan Error, ctx: Context }

worker:
  for req in requests {
    res := driver.execute_query(conn, req.query, len(req.params), req.params)
    if res.error != OK { close(req.rows); req.done <- err; continue }
    for {
      select {
        case <-req.ctx.done: driver.cancel(conn); break
        default:
      }
      p := driver.fetch_next(&res)
      if p.data == null { break }
      req.rows <- Row{ payload: &p }
    }
    close(req.rows)
    req.done <- nil
    driver.clear_result(&res)
  }

If building on libpq async:
- Use driver.get_event_fd(conn) with OS poll/select.
- Loop while driver.poll_ready(conn) == 0; respect ctx timeout/cancel.
- When driver.result_ready(conn) == 1, call driver.get_result(conn) and stream rows.
This preserves zero-copy: payload is valid until clear_result().
*/
