use warp::ws::{WebSocket, Message};
use crate::aelira::AeliraRef;
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

pub async fn handle_socket(ws: WebSocket, client_name: String, user_id: String, aelira: AeliraRef) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let session_id = {
        let mut manager = aelira.sessions.lock().unwrap();
        let session = manager.create(user_id.clone(), client_name.clone(), tx);
        session.id.clone()
    };

    println!("connected: {} (ID: {}) session: {}", client_name, user_id, session_id);
  
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    let ready = json!({
        "op": "ready",
        "resumed": false,
        "sessionId": session_id
    });

    if let Err(e) = user_ws_tx.send(Message::text(ready.to_string())).await {
        eprintln!("Failed to send ready op: {}", e);
        return;
    }

    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                if let Err(e) = user_ws_tx.send(msg).await {
                    eprintln!("Error sending to websocket: {}", e);
                    break;
                }
            }
            result = user_ws_rx.next() => {
                match result {
                    Some(Ok(msg)) => {
                        if msg.is_close() {
                            println!("WebSocket closed by client: {}, session: {}", client_name, session_id);
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Websocket error: {}", e);
                        break;
                    }
                    None => break,
                }
            }
        }
    }
}