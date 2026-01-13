use crate::playback::voice::{crypto::VoiceCrypto, udp::VoiceUdp, websocket::VoiceWebsocket};
use crate::utils::{log, Level};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

const OPUS_SILENCE_FRAME: [u8; 3] = [0xF8, 0xFF, 0xFE];

pub struct VoiceConnection {
    pub guild_id: String,
    pub session_id: String,
    pub token: String,
    pub endpoint: String,
    pub user_id: String,
    pub crypto: Arc<Mutex<Option<VoiceCrypto>>>,
    pub udp: Arc<Mutex<Option<VoiceUdp>>>,
    pub sender: mpsc::UnboundedSender<Message>,
    receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<Message>>>>,
    pub ssrc: Arc<Mutex<u32>>,
    pub speaking: Arc<Mutex<bool>>,
}

#[derive(Deserialize)]
struct VoiceOp {
    pub op: u8,
    pub d: serde_json::Value,
}

impl VoiceConnection {
    pub fn new(guild_id: String, session_id: String, token: String, endpoint: String, user_id: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            guild_id,
            session_id,
            token,
            endpoint,
            user_id,
            crypto: Arc::new(Mutex::new(None)),
            udp: Arc::new(Mutex::new(None)),
            sender: tx,
            receiver: Arc::new(Mutex::new(Some(rx))),
            ssrc: Arc::new(Mutex::new(0)),
            speaking: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn set_speaking(&self, speaking: bool) {
        let ssrc = {
            let lock = self.ssrc.lock().await;
            *lock
        };

        let payload = json!({
            "op": 5,
            "d": {
                "speaking": if speaking { 1 } else { 0 },
                "delay": 0,
                "ssrc": ssrc
            }
        });

        {
            let mut s = self.speaking.lock().await;
            *s = speaking;
        }

        let _ = self.sender.send(Message::Text(payload.to_string().into()));
    }

    pub async fn send_silence(&self) {
        let udp_lock = self.udp.lock().await;
        let crypto_lock = self.crypto.lock().await;

        if let (Some(mut udp), Some(crypto)) = (udp_lock.clone(), crypto_lock.clone()) {
            for _ in 0..5 {
                udp.send_opus(&OPUS_SILENCE_FRAME, &crypto).await;
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
    }

    pub async fn run(&self) {
        let url = format!("wss://{}/?v=8", self.endpoint);
        log(Level::Debug, "Voice", format!("Connecting to voice WS: {}", url));

        let (ws_stream, _) = match connect_async(&url).await {
            Ok(v) => v,
            Err(e) => {
                log(Level::Error, "Voice", format!("Failed to connect voice WS: {}", e));
                return;
            }
        };

        log(Level::Info, "Voice", "Voice WS Connected");

        let (mut ws_write, mut ws_read) = ws_stream.split();
        let mut rx = {
            let mut lock = self.receiver.lock().await;
            lock.take().unwrap()
        };

        VoiceWebsocket::identify(&mut ws_write, &self.guild_id, &self.user_id, &self.session_id, &self.token).await;

        let mut heartbeat_interval = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                _ = heartbeat_interval.tick() => {
                    let payload = json!({
                        "op": 3,
                        "d": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
                    });
                    let _ = ws_write.send(Message::Text(payload.to_string().into())).await;
                }
                Some(msg) = rx.recv() => {
                    let text = msg.to_text().unwrap_or("").to_string();
                    let _ = ws_write.send(tokio_tungstenite::tungstenite::Message::Text(text.into())).await;
                }
                Some(msg_res) = ws_read.next() => {
                    match msg_res {
                        Ok(msg) => {
                            if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                                let op: VoiceOp = serde_json::from_str(&text).unwrap();
                                match op.op {
                                    2 => {
                                        let ip = op.d["ip"].as_str().unwrap();
                                        let port = op.d["port"].as_u64().unwrap() as u16;
                                        let ssrc = op.d["ssrc"].as_u64().unwrap() as u32;

                                        {
                                            let mut s = self.ssrc.lock().await;
                                            *s = ssrc;
                                        }

                                        let addr = format!("{}:{}", ip, port).parse().unwrap();
                                        let udp = VoiceUdp::new(addr, ssrc).await;
                                        let (ext_ip, ext_port) = udp.discover_ip().await;

                                        log(Level::Debug, "Voice", format!("UDP Socket ready, IP discovered: {}:{}", ext_ip, ext_port));

                                        let mut udp_lock = self.udp.lock().await;
                                        *udp_lock = Some(udp);
                                        drop(udp_lock);

                                        VoiceWebsocket::select_protocol(&mut ws_write, &ext_ip, ext_port).await;
                                    },
                                    4 => {
                                        let key = op.d["secret_key"].as_array().unwrap()
                                            .iter().map(|v| v.as_u64().unwrap() as u8).collect::<Vec<u8>>();

                                        let mut crypto_lock = self.crypto.lock().await;
                                        *crypto_lock = Some(VoiceCrypto::new(&key));
                                        drop(crypto_lock);

                                        log(Level::Info, "Voice", "Voice crypto setup complete");
                                    },
                                    8 => {
                                        let interval_ms = op.d["heartbeat_interval"].as_u64().unwrap();
                                        heartbeat_interval = interval(Duration::from_millis(interval_ms));
                                    },
                                    _ => {}
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
                else => break,
            }
        }

        log(Level::Info, "Voice", "Voice WS Loop Ended");
    }
}