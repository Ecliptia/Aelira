use crate::playback::processor::AudioProcessor;
use crate::playback::voice::connection::VoiceConnection;
use crate::playback::voice::stream::AudioStream;
use crate::sources::local::LocalSource;
use crate::utils::encoding::DecodedInfo;
use crate::utils::{log, Level};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrackData {
    pub encoded: String,
    pub info: DecodedInfo,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    pub time: u64,
    pub position: i64,
    pub connected: bool,
    pub ping: i64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VoiceState {
    pub token: String,
    pub endpoint: String,
    pub session_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub guild_id: String,
    pub track: Option<TrackData>,
    pub volume: u16,
    pub paused: bool,
    pub state: PlayerState,
    pub voice: Option<VoiceState>,
    #[serde(skip)]
    pub connection: Option<Arc<VoiceConnection>>,
}

impl Player {
    pub fn new(guild_id: String) -> Self {
        Self {
            guild_id,
            track: None,
            volume: 100,
            paused: false,
            state: PlayerState {
                time: 0,
                position: 0,
                connected: false,
                ping: -1,
            },
            voice: None,
            connection: None,
        }
    }

    pub fn connect(&mut self, voice: VoiceState, user_id: String) {
        log(Level::Info, "Player", format!("Connecting to voice: {} (Session: {})", voice.endpoint, voice.session_id));

        let conn = Arc::new(VoiceConnection::new(
            self.guild_id.clone(),
            voice.session_id.clone(),
            voice.token.clone(),
            voice.endpoint.clone(),
            user_id,
        ));

        self.connection = Some(conn.clone());
        self.voice = Some(voice);

        tokio::spawn(async move {
            conn.run().await;
        });
    }

    pub fn play(&mut self) {
        let track = match &self.track {
            Some(t) => t,
            None => {
                log(Level::Warn, "Player", "Attempted to play without track");
                return;
            },
        };

        log(Level::Debug, "Player", format!("Play request for track: {}", track.info.identifier));

        if let Some(conn) = &self.connection {
            let conn_arc = conn.clone();
            let identifier = track.info.identifier.clone();

            tokio::spawn(async move {
                let (udp, crypto) = {
                    let mut attempts = 0;
                    loop {
                        let udp_lock = conn_arc.udp.lock().await;
                        let crypto_lock = conn_arc.crypto.lock().await;

                        if let (Some(udp), Some(crypto)) = (udp_lock.as_ref(), crypto_lock.as_ref()) {
                            let udp_final = Arc::new(Mutex::new(udp.clone()));
                            let crypto_final = Arc::new(crypto.clone());
                            break (udp_final, crypto_final);
                        }

                        drop(udp_lock);
                        drop(crypto_lock);

                        if attempts > 50 {
                            log(Level::Error, "Player", "Timeout waiting for voice connection");
                            return;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        attempts += 1;
                    }
                };

                let stream_handler = AudioStream::new(udp, crypto);
                use crate::managers::sources::Source;

                match LocalSource.load_stream(&identifier).await {
                    Some(framed) => {
                        log(Level::Info, "Player", format!("Stream loaded for: {}", identifier));
                        conn_arc.set_speaking(true).await;

                        let processor: AudioProcessor<tokio::fs::File> = AudioProcessor::new(framed.into_inner(), "webm/opus").await;
                        let source_stream = stream::unfold(processor, |mut proc: AudioProcessor<tokio::fs::File>| async move {
                            match proc.next_packet().await {
                                Some(Ok(packet)) => Some((Ok(packet), proc)),
                                Some(Err(e)) => {
                                    log(Level::Error, "Player", format!("Error reading packet: {}", e));
                                    Some((Err(e), proc))
                                },
                                None => None,
                            }
                        });

                        stream_handler.play(Box::pin(source_stream)).await;
                        conn_arc.set_speaking(false).await;
                        conn_arc.send_silence().await;
                        log(Level::Info, "Player", "Playback finished");
                    },
                    None => {
                        log(Level::Error, "Player", format!("Failed to load stream for: {}", identifier));
                    }
                }
            });
        } else {
            log(Level::Warn, "Player", "No active voice connection to play on");
        }
    }
}

pub struct PlayerManager {
    pub players: HashMap<String, Player>,
}

impl PlayerManager {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, guild_id: String) -> &mut Player {
        self.players.entry(guild_id.clone()).or_insert_with(|| Player::new(guild_id))
    }
}