use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use warp::ws::Message;
use rand::{distr::Alphanumeric, Rng};
use crate::managers::players::PlayerManager;

pub struct Session {
    pub id: String,
    pub user_id: String,
    pub _client_name: String,
    pub sender: Mutex<mpsc::UnboundedSender<Message>>,
    pub players: Mutex<PlayerManager>,
}

pub struct SessionManager {
    pub sessions: HashMap<String, Arc<Session>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create(
        &mut self,
        user_id: String,
        client_name: String,
        sender: mpsc::UnboundedSender<Message>,
    ) -> Arc<Session> {
        let id: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let session = Arc::new(Session {
            id: id.clone(),
            user_id,
            _client_name: client_name,
            sender: Mutex::new(sender),
            players: Mutex::new(PlayerManager::new()),
        });

        self.sessions.insert(id, session.clone());
        session
    }

    pub fn resume(&self, session_id: &str, new_sender: mpsc::UnboundedSender<Message>) -> Option<Arc<Session>> {
        if let Some(session) = self.sessions.get(session_id) {
            let mut sender = session.sender.lock().unwrap();
            *sender = new_sender;
            return Some(session.clone());
        }
        None
    }
}
