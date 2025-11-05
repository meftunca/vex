
# Pro Extensions Added

This archive augments the original `vex_async_runtime` with minimal API surface for:
- Timers: `worker_await_deadline`, `worker_await_after`
- Cancellation: `CancelToken`, `worker_cancel_token`, `cancel_requested`, `cancel_request`
- Generic IO handle: `IoHandle` and `worker_await_ioh`
- Auto-shutdown toggle: `runtime_enable_auto_shutdown`
- Stats: `RuntimeStats` and `runtime_get_stats`

> Note: Implementations are lightweight reference stubs meant to illustrate the integration points for Vex/LLVM IR. You may replace internals with production-grade algorithms (timer wheel, work stealing, parking/unparking) without breaking the C ABI.
