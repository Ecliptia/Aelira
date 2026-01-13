use std::time::Duration;
use tokio::time::{interval, MissedTickBehavior};
use std::sync::Arc;
use tokio::sync::Mutex;
use futures_util::StreamExt;
use crate::playback::voice::udp::VoiceUdp;
use crate::playback::voice::crypto::VoiceCrypto;
use crate::utils::{log, Level};

const FRAME_DURATION: u64 = 20;

pub struct AudioStream {
    udp: Arc<Mutex<VoiceUdp>>,
    crypto: Arc<VoiceCrypto>,
}

impl AudioStream {
    pub fn new(udp: Arc<Mutex<VoiceUdp>>, crypto: Arc<VoiceCrypto>) -> Self {
        Self { udp, crypto }
    }

    pub async fn play<S>(&self, mut source: S)
    where
        S: StreamExt<Item = Result<Vec<u8>, std::io::Error>> + Unpin + Send + 'static,
    {
        let mut ticker = interval(Duration::from_millis(FRAME_DURATION));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Burst);
        let mut count = 0;

        loop {
            ticker.tick().await;

            match source.next().await {
                Some(Ok(frame)) => {
                    let mut udp = self.udp.lock().await;
                    udp.send_opus(&frame, &self.crypto).await;
                    count += 1;
                    if count % 500 == 0 {
                        log(Level::Debug, "AudioStream", format!("Sent {} frames", count));
                    }
                }
                Some(Err(e)) => {
                    log(Level::Error, "AudioStream", format!("Error reading frame: {}", e));
                    break;
                },
                None => {
                    log(Level::Debug, "AudioStream", "Source reached EOF");
                    break;
                },
            }
        }
    }
}