use crate::aelira::AeliraRef;
use crate::utils::{log, Level};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use warp::ws::{Message, WebSocket};

pub async fn handle_socket(ws: WebSocket, client_name: String, user_id: String, session_id_header: Option<String>, aelira: AeliraRef) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut resumed = false;

    let session_id = {
        let mut manager = aelira.sessions.lock().unwrap();
        let existing_session = if let Some(id) = session_id_header {
            manager.resume(&id, tx.clone())
        } else {
            None
        };

        if let Some(session) = existing_session {
            resumed = true;
            session.id.clone()
        } else {
            let session = manager.create(user_id.clone(), client_name.clone(), tx);
            session.id.clone()
        }
    };

    log(Level::Info, "Socket", format!("{} connected: {} ({}) session: {}", if resumed { "Resumed" } else { "New" }, client_name, user_id, session_id));

    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let ready = json!({
        "op": "ready",
        "resumed": resumed,
        "sessionId": session_id
    });

    if let Err(e) = user_ws_tx.send(Message::text(ready.to_string())).await {
        log(Level::Error, "Socket", format!("Failed to send ready op to {}: {}", session_id, e));
        return;
    }

    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                if let Err(e) = user_ws_tx.send(msg).await {
                    log(Level::Debug, "Socket", format!("Error sending to websocket {}: {}", session_id, e));
                    break;
                }
            }
            result = user_ws_rx.next() => {
                match result {
                    Some(Ok(msg)) => {
                        if msg.is_close() {
                            log(Level::Info, "Socket", format!("WebSocket closed by client: {}, session: {}", client_name, session_id));
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        log(Level::Error, "Socket", format!("Websocket error for {}: {}", session_id, e));
                        break;
                    }
                    None => break,
                }
            }
        }
    }
}