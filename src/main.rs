mod api;
mod aelira;
mod config;

use std::fs;
use config::Config;
use warp::Filter;

fn main() {
    let config = Config::load().expect("Failed to load configuration");

    let workers = if let Some(cluster) = &config.cluster {
        match cluster.workers.unwrap_or(0) {
            0 => std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
            n => n,
        }
    } else {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
    };

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main(config));
}

async fn async_main(config: Config) {
    let version = fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml")
        .lines()
        .find(|line| line.starts_with("version = "))
        .and_then(|line| line.split('"').nth(1))
        .unwrap_or("unknown")
        .to_string();

    let aelira = std::sync::Arc::new(aelira::Aelira::new(&config, version));
    
    let addr: std::net::SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Invalid address");
    
    let aelira_clone = aelira.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            {
                let mut sys = aelira_clone.system.lock().unwrap();
                sys.refresh_cpu_usage();
                sys.refresh_memory();
            }
        }
    });

    let routes = api::routes::all_routes(aelira.clone())
        .recover(api::handle_rejection);

    warp::serve(routes)
        .run(addr)
        .await;
}