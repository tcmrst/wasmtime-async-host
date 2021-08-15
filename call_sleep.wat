(module
    (import "host" "sleep_async" (func $host_sleep_async (param i64)))
    (import "host" "sleep_sync" (func $host_sleep_sync (param i64)))
    (func (export "run-async") 
        i64.const 1
        call $host_sleep_async
    )
    (func (export "run-sync") 
        i64.const 1
        call $host_sleep_sync
    )
)
