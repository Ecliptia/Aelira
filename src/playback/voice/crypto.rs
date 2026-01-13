use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::Aead};

#[derive(Clone)]
pub struct VoiceCrypto {
    cipher: Aes256Gcm,
}

impl VoiceCrypto {
    pub fn new(secret_key: &[u8]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(secret_key);
        Self {
            cipher: Aes256Gcm::new(key),
        }
    }

    pub fn encrypt(&self, packet: &[u8], nonce: &[u8], ad: &[u8]) -> Vec<u8> {
        let nonce = Nonce::from_slice(nonce);
        self.cipher.encrypt(nonce, aes_gcm::aead::Payload { msg: packet, aad: ad }).unwrap()
    }
}