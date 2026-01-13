pub mod api;
pub mod aelira;
pub mod config;
pub mod socket;
pub mod managers;
pub mod utils;
pub mod playback;
pub mod sources;
pub mod models;

use std::fs;
use config::Config;
use warp::Filter;
use utils::{log, Level};

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

            let mut total_players = 0;
            let mut playing_players = 0;
            {
                let sessions = aelira_clone.sessions.lock().unwrap();
                for session in sessions.sessions.values() {
                    let players = session.players.lock().unwrap();
                    total_players += players.players.len() as u32;
                    for player in players.players.values() {
                        if !player.paused && player.track.is_some() {
                            playing_players += 1;
                        }
                    }
                }
            }

            aelira_clone.stats.set_players(total_players);
            aelira_clone.stats.set_playing_players(playing_players);

            let stats_payload = {
                let sys = aelira_clone.system.lock().unwrap();
                serde_json::json!({
                    "op": "stats",
                    "players": total_players,
                    "playingPlayers": playing_players,
                    "uptime": sysinfo::System::uptime() * 1000,
                    "memory": {
                        "free": sys.free_memory(),
                        "used": sys.used_memory(),
                        "allocated": sys.used_memory(),
                        "reservable": sys.total_memory(),
                    },
                    "cpu": {
                        "cores": sys.cpus().len(),
                        "systemLoad": sys.global_cpu_usage(),
                        "aeliraLoad": 0.0,
                    }
                }).to_string()
            };

            let sessions = aelira_clone.sessions.lock().unwrap();
            for session in sessions.sessions.values() {
                {
                    let sender = session.sender.lock().unwrap();
                    let _ = sender.send(warp::ws::Message::text(&stats_payload));
                }

                let players = session.players.lock().unwrap();
                for player in players.players.values() {
                    if player.track.is_some() {
                        let update = serde_json::json!({
                            "op": "playerUpdate",
                            "guildId": player.guild_id,
                            "state": {
                                "time": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                                "position": player.state.position,
                                "connected": player.connection.is_some(),
                                "ping": player.state.ping
                            }
                        }).to_string();
                        let sender = session.sender.lock().unwrap();
                        let _ = sender.send(warp::ws::Message::text(&update));
                    }
                }
            }
        }
    });

    let routes = api::routes::all_routes(aelira.clone())
        .recover(api::handle_rejection);

    log(Level::Info, "Server", format!("Aelira v{} started on http://{}", aelira.version, addr));

    warp::serve(routes)
        .run(addr)
        .await;
}