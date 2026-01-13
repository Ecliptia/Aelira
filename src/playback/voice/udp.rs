use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::sync::Arc;
use crate::playback::voice::crypto::VoiceCrypto;

#[derive(Clone)]
pub struct VoiceUdp {
    pub socket: Arc<UdpSocket>,
    pub destination: SocketAddr,
    pub ssrc: u32,
    pub sequence: u16,
    pub timestamp: u32,
    pub nonce: u32,
}

impl VoiceUdp {
    pub async fn new(addr: SocketAddr, ssrc: u32) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        Self {
            socket: Arc::new(socket),
            destination: addr,
            ssrc,
            sequence: 0,
            timestamp: 0,
            nonce: 0,
        }
    }

    pub async fn discover_ip(&self) -> (String, u16) {
        let mut packet = [0u8; 74];
        packet[0..2].copy_from_slice(&1u16.to_be_bytes());
        packet[2..4].copy_from_slice(&70u16.to_be_bytes());
        packet[4..8].copy_from_slice(&self.ssrc.to_be_bytes());

        self.socket.send_to(&packet, self.destination).await.unwrap();
        let mut res = [0u8; 74];
        let (len, _) = self.socket.recv_from(&mut res).await.unwrap();

        let ip = std::str::from_utf8(&res[8..len - 2]).unwrap().trim_matches(char::from(0)).to_string();
        let port = u16::from_be_bytes([res[len - 2], res[len - 1]]);

        (ip, port)
    }

    pub async fn send_opus(&mut self, payload: &[u8], crypto: &VoiceCrypto) {
        let mut header = [0u8; 12];
        header[0] = 0x80;
        header[1] = 0x78;
        header[2..4].copy_from_slice(&self.sequence.to_be_bytes());
        header[4..8].copy_from_slice(&self.timestamp.to_be_bytes());
        header[8..12].copy_from_slice(&self.ssrc.to_be_bytes());

        let mut nonce = [0u8; 12];
        nonce[0..4].copy_from_slice(&self.nonce.to_be_bytes());

        let encrypted = crypto.encrypt(payload, &nonce, &header);
        
        let mut packet = Vec::with_capacity(12 + encrypted.len() + 4);
        packet.extend_from_slice(&header);
        packet.extend_from_slice(&encrypted);
        packet.extend_from_slice(&self.nonce.to_be_bytes());

        let _ = self.socket.send_to(&packet, self.destination).await;

        self.sequence = self.sequence.wrapping_add(1);
        self.timestamp = self.timestamp.wrapping_add(960);
        self.nonce = self.nonce.wrapping_add(1);
    }
}
