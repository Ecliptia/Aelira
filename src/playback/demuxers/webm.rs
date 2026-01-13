use bytes::{Buf, BytesMut, Bytes};
use tokio_util::codec::Decoder;
use std::io;

const OPUS_HEAD: &[u8] = b"OpusHead";

const EBML_HEADER: u64 = 0x1A45DFA3;
const SEGMENT: u64 = 0x18538067;
const CLUSTER: u64 = 0x1F43B675;
const TRACKS: u64 = 0x1654AE6B;
const TRACK_ENTRY: u64 = 0xAE;
const TRACK_NUMBER: u64 = 0xD7;
const TRACK_TYPE: u64 = 0x83;
const SIMPLE_BLOCK: u64 = 0xA3;
const CODEC_PRIVATE: u64 = 0x63A2;
const VOID: u64 = 0xEC;

#[derive(Default)]
pub struct WebmOpusDemuxer {
    current_track_number: Option<u64>,
    pending_track_number: u64,
    pending_track_type: u64,
    skip_len: usize,
}

impl WebmOpusDemuxer {
    pub fn new() -> Self {
        Self::default()
    }

    fn read_vint(buf: &[u8], keep_marker: bool) -> Option<(u64, usize)> {
        if buf.is_empty() { return None; }
        
        let first_byte = buf[0];
        let width = first_byte.leading_zeros() as usize + 1;
        
        if width > 8 || buf.len() < width { return None; }

        let mut val = if keep_marker {
            first_byte as u64
        } else {
            (first_byte & ((1 << (8 - width)) - 1)) as u64
        };

        for i in 1..width {
            val = (val << 8) | (buf[i] as u64);
        }

        Some((val, width))
    }
}

impl Decoder for WebmOpusDemuxer {
    type Item = Bytes;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            if self.skip_len > 0 {
                if src.len() >= self.skip_len {
                    src.advance(self.skip_len);
                    self.skip_len = 0;
                } else {
                    self.skip_len -= src.len();
                    src.clear();
                    return Ok(None);
                }
            }

            let (id, id_len) = match Self::read_vint(src, true) {
                Some(v) => v,
                None => return Ok(None),
            };

            let (size, size_len) = match Self::read_vint(&src[id_len..], false) {
                Some(v) => (v.0 as usize, v.1),
                None => return Ok(None),
            };

            let total_header_len = id_len + size_len;

            println!("[Demuxer] ID: {:X}, Size: {}, ID Len: {}, Size Len: {}", id, size, id_len, size_len);

            match id {
                EBML_HEADER | SEGMENT | CLUSTER | TRACKS | TRACK_ENTRY => {
                    println!("[Demuxer] Entering container: {:X}", id);
                    src.advance(total_header_len);
                    if id == TRACK_ENTRY {
                        self.pending_track_number = 0;
                        self.pending_track_type = 0;
                    }
                    continue;
                },
                TRACK_NUMBER => {
                    if src.len() < total_header_len + size { return Ok(None); }
                    src.advance(total_header_len);
                    
                    let mut num = 0u64;
                    for i in 0..size {
                        num = (num << 8) | (src[i] as u64);
                    }
                    self.pending_track_number = num;
                    println!("[Demuxer] Pending Track Number: {}", num);
                    
                    src.advance(size); 
                },
                TRACK_TYPE => {
                    if src.len() < total_header_len + size { return Ok(None); }
                    src.advance(total_header_len);
                    
                    let mut typ = 0u64;
                    for i in 0..size {
                        typ = (typ << 8) | (src[i] as u64);
                    }
                    self.pending_track_type = typ;
                    println!("[Demuxer] Pending Track Type: {}", typ);
                    
                    if self.pending_track_type == 2 { 
                        self.current_track_number = Some(self.pending_track_number);
                        println!("[Demuxer] Successfully selected Audio Track: {}", self.pending_track_number);
                    }
                    src.advance(size);
                },
                CODEC_PRIVATE => {
                    if src.len() < total_header_len + size { return Ok(None); }
                    src.advance(total_header_len);
                    if size >= 8 && &src[0..8] == OPUS_HEAD {
                        println!("[Demuxer] Found Opus Private Data Header");
                    }
                    src.advance(size);
                },
                SIMPLE_BLOCK => {
                    if src.len() < total_header_len + size { return Ok(None); }
                    src.advance(total_header_len);
                    
                    let (track_num, track_len) = match Self::read_vint(src, false) {
                        Some(v) => v,
                        None => { 
                            println!("[Demuxer] Failed to read track number in SimpleBlock");
                            src.advance(size); 
                            continue; 
                        }
                    };

                    if Some(track_num) == self.current_track_number {
                        let header_skip = track_len + 3;
                        let payload = src.copy_to_bytes(size).slice(header_skip..);
                        // println!("[Demuxer] Extracted frame: {} bytes", payload.len());
                        return Ok(Some(payload));
                    } else {
                        src.advance(size);
                    }
                },
                VOID => {
                     println!("[Demuxer] Skipping Void element");
                     src.advance(total_header_len);
                     self.skip_len = size;
                },
                _ => {
                    println!("[Demuxer] Unknown ID: {:X}, skipping {} bytes", id, size);
                    src.advance(total_header_len);
                    self.skip_len = size;
                }
            }
        }
    }
}