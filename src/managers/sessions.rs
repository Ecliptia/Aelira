use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use warp::ws::Message;
use rand::{distr::Alphanumeric, Rng};

pub struct Session {
    pub id: String,
    pub user_id: String,
    pub client_name: String,
    pub sender: mpsc::UnboundedSender<Message>,
}

pub struct SessionManager {
    sessions: HashMap<String, Arc<Session>>,
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
            client_name,
            sender,
        });

        self.sessions.insert(id, session.clone());
        session
    }
}