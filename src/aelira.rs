use std::sync::{Arc, Mutex};
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use crate::managers::sessions::SessionManager;

pub struct Aelira {
    pub version: String,
    pub password: Option<String>,
    pub system: Arc<Mutex<System>>,
    pub sessions: Mutex<SessionManager>,
}

impl Aelira {
    pub fn new(config: &crate::config::Config, version: String) -> Self {
        let refresh = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            .with_memory(MemoryRefreshKind::nothing().with_ram());
            
        let system = System::new_with_specifics(refresh);

        Aelira {
            version,
            password: config.server.password.clone(),
            system: Arc::new(Mutex::new(system)),
            sessions: Mutex::new(SessionManager::new()),
        }
    }
}

pub type AeliraRef = Arc<Aelira>;