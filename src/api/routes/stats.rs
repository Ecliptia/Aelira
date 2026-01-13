use warp::Filter;
use serde::Serialize;
use sysinfo::System;
use crate::aelira::AeliraRef;

#[derive(Serialize)]
struct MemoryStats {
    free: u64,
    used: u64,
    allocated: u64,
    reservable: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CpuStats {
    cores: usize,
    system_load: f32,
    aelira_load: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatsResponse {
    players: u32,
    playing_players: u32,
    uptime: u64,
    memory: MemoryStats,
    cpu: CpuStats,
    frame_stats: Option<()>,
}

pub fn handler(aelira: AeliraRef) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let with_aelira = warp::any().map(move || aelira.clone());
    warp::get()
        .and(with_aelira)
        .map(|aelira: AeliraRef| {
            let sys = aelira.system.lock().unwrap();
            let memory = MemoryStats {
                free: sys.free_memory(),
                used: sys.used_memory(),
                allocated: sys.used_memory(),
                reservable: sys.total_memory(),
            };
            let cpu = CpuStats {
                cores: sys.cpus().len(),
                system_load: sys.global_cpu_usage(),
                aelira_load: 0.0,
            };
            let stats = StatsResponse {
                players: aelira.stats.players.load(std::sync::atomic::Ordering::Relaxed),
                playing_players: aelira.stats.playing_players.load(std::sync::atomic::Ordering::Relaxed),
                uptime: System::uptime() * 1000,
                memory,
                cpu,
                frame_stats: None,
            };
            warp::reply::json(&stats)
        })
}