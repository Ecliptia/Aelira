use tokio_tungstenite::tungstenite::protocol::Message;
use serde_json::json;
use futures_util::{Sink, SinkExt};

pub struct VoiceWebsocket;

impl VoiceWebsocket {
    pub async fn identify<S>(write: &mut S, guild_id: &str, user_id: &str, session_id: &str, token: &str) 
    where S: Sink<Message> + Unpin, <S as Sink<Message>>::Error: std::fmt::Debug
    {
        let identify = json!({
            "op": 0,
            "d": {
                "server_id": guild_id,
                "user_id": user_id,
                "session_id": session_id,
                "token": token,
            }
        });
        let _ = write.send(Message::Text(identify.to_string().into())).await;
    }

    pub async fn select_protocol<S>(write: &mut S, ip: &str, port: u16)
    where S: Sink<Message> + Unpin, <S as Sink<Message>>::Error: std::fmt::Debug
    {
        let select = json!({
            "op": 1,
            "d": {
                "protocol": "udp",
                "data": {
                    "address": ip,
                    "port": port,
                    "mode": "aead_aes256_gcm_rtpsize"
                }
            }
        });
        let _ = write.send(Message::Text(select.to_string().into())).await;
    }
}
