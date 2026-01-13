use std::sync::{Arc, Mutex};
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use crate::managers::sessions::SessionManager;
use crate::managers::sources::SourceManager;
use crate::managers::stats::StatsManager;
use crate::managers::route_planner::RoutePlannerManager;
use crate::sources::local::LocalSource;

pub struct Aelira {
    pub version: String,
    pub password: Option<String>,
    pub system: Arc<Mutex<System>>,
    pub sessions: Mutex<SessionManager>,
    pub sources: Arc<SourceManager>,
    pub stats: Arc<StatsManager>,
    pub route_planner: Arc<RoutePlannerManager>,
}

impl Aelira {
    pub fn new(config: &crate::config::Config, version: String) -> Self {
        let refresh = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            .with_memory(MemoryRefreshKind::nothing().with_ram());
            
        let system = System::new_with_specifics(refresh);
        let mut sources = SourceManager::new();
        sources.register(Box::new(LocalSource));

        Aelira {
            version,
            password: config.server.password.clone(),
            system: Arc::new(Mutex::new(system)),
            sessions: Mutex::new(SessionManager::new()),
            sources: Arc::new(sources),
            stats: Arc::new(StatsManager::new()),
            route_planner: Arc::new(RoutePlannerManager::new()),
        }
    }
}

pub type AeliraRef = Arc<Aelira>;