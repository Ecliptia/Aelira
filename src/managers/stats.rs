use std::sync::atomic::{AtomicU32, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct StatsManager {
    pub api_requests: Mutex<HashMap<String, AtomicU32>>,
    pub _api_errors: Mutex<HashMap<String, AtomicU32>>,
    pub players: AtomicU32,
    pub playing_players: AtomicU32,
}

impl StatsManager {
    pub fn new() -> Self {
        Self {
            api_requests: Mutex::new(HashMap::new()),
            _api_errors: Mutex::new(HashMap::new()),
            players: AtomicU32::new(0),
            playing_players: AtomicU32::new(0),
        }
    }

    pub fn increment_api_request(&self, endpoint: &str) {
        let mut map = self.api_requests.lock().unwrap();
        let counter = map.entry(endpoint.to_string()).or_insert_with(|| AtomicU32::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn _increment_api_error(&self, endpoint: &str) {
        let mut map = self._api_errors.lock().unwrap();
        let counter = map.entry(endpoint.to_string()).or_insert_with(|| AtomicU32::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_players(&self, count: u32) {
        self.players.store(count, Ordering::Relaxed);
    }

    pub fn set_playing_players(&self, count: u32) {
        self.playing_players.store(count, Ordering::Relaxed);
    }
    
    pub fn _get_api_stats(&self) -> HashMap<String, u32> {
        let map = self.api_requests.lock().unwrap();
        map.iter().map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed))).collect()
    }
}
