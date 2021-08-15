use std::time::Duration;
use tokio::runtime::Builder;
use wasmtime::{Config, Engine, Linker, Module, Store};

const WORKER_THREADS: usize = 2;
const TASKS: usize = 5;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("usage: {} funcname", args[0]);
        return Ok(());
    }

    let runtime = Builder::new_multi_thread()
        .worker_threads(WORKER_THREADS)
        .enable_time()
        .build()?;

    let engine = Engine::new(Config::new().async_support(true))?;
    let module = Module::from_file(&engine, "call_sleep.wat")?;
    let mut linker = Linker::new(&engine);

    linker.func_wrap1_async("host", "sleep_async", |_caller, secs: u64| {
        Box::new(async move {
            println!("start hostfunc");
            tokio::time::sleep(Duration::from_secs(secs)).await;
            println!("end hostfunc");
            Ok(())
        })
    })?;
    linker.func_wrap("host", "sleep_sync", |secs: u64| {
        println!("start hostfunc");
        std::thread::sleep(Duration::from_secs(secs));
        println!("end hostfunc");
        Ok(())
    })?;

    let join_handles = (0..TASKS).map(|_| {
        let mut store = Store::new(&engine, ());
        let module = module.clone();
        let linker = linker.clone();
        let funcname = args[1].clone();

        runtime.spawn(async move {
            let instance = linker
                .instantiate_async(&mut store, &module).await.unwrap();
            let func = instance
                .get_typed_func::<(), (), _>(&mut store, &funcname)
                .unwrap();

            func.call_async(&mut store, ()).await.unwrap();
        })
    });

    runtime.block_on(futures::future::join_all(join_handles));
    Ok(())
}
