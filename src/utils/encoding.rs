use bytes::{Buf, BufMut, BytesMut};
use base64::{Engine as _, engine::general_purpose};
use std::io::Cursor;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecodedInfo {
    pub title: String,
    pub author: String,
    pub length: u64,
    pub identifier: String,
    pub is_stream: bool,
    pub uri: Option<String>,
    pub artwork_url: Option<String>,
    pub isrc: Option<String>,
    pub source_name: String,
    pub position: u64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecodedTrack {
    pub encoded: String,
    pub info: DecodedInfo,
    pub plugin_info: serde_json::Value,
    pub user_data: serde_json::Value,
}

pub fn decode_track(encoded: &str) -> Result<DecodedTrack, Box<dyn std::error::Error>> {
    let bytes = general_purpose::STANDARD.decode(encoded)?;
    let mut cursor = Cursor::new(bytes);

    let header = cursor.get_i32();
    let size = (header & 0x3fffffff) as usize;
    
    let mut data = vec![0u8; size];
    cursor.copy_to_slice(&mut data);
    let mut msg = Cursor::new(data);

    let version = msg.get_u8();
    let title = read_utf(&mut msg)?;
    let author = read_utf(&mut msg)?;
    let length = msg.get_u64();
    let identifier = read_utf(&mut msg)?;
    let is_stream = msg.get_u8() != 0;
    
    let uri = if version >= 2 { read_nullable_text(&mut msg)? } else { None };
    let artwork_url = if version >= 3 { read_nullable_text(&mut msg)? } else { None };
    let isrc = if version >= 3 { read_nullable_text(&mut msg)? } else { None };
    
    let source_name = read_utf(&mut msg)?;
    let position = msg.get_u64();

    Ok(DecodedTrack {
        encoded: encoded.to_string(),
        info: DecodedInfo {
            title,
            author,
            length,
            identifier,
            is_stream,
            uri,
            artwork_url,
            isrc,
            source_name,
            position,
        },
        plugin_info: serde_json::json!({}),
        user_data: serde_json::json!({}),
    })
}

pub fn encode_track(info: &DecodedInfo) -> String {
    let mut buf = BytesMut::new();
    let version = if info.artwork_url.is_some() || info.isrc.is_some() { 3 } else if info.uri.is_some() { 2 } else { 1 };
    
    buf.put_u8(version);
    write_utf(&mut buf, &info.title);
    write_utf(&mut buf, &info.author);
    buf.put_u64(info.length);
    write_utf(&mut buf, &info.identifier);
    buf.put_u8(if info.is_stream { 1 } else { 0 });
    
    if version >= 2 {
        write_nullable_text(&mut buf, info.uri.as_deref());
    }
    if version >= 3 {
        write_nullable_text(&mut buf, info.artwork_url.as_deref());
        write_nullable_text(&mut buf, info.isrc.as_deref());
    }
    
    write_utf(&mut buf, &info.source_name);
    buf.put_u64(info.position);

    let header = (buf.len() as u32 & 0x3fffffff) | (1 << 30);
    let mut res = BytesMut::with_capacity(4 + buf.len());
    res.put_u32(header);
    res.put(buf);
    
    general_purpose::STANDARD.encode(res)
}

fn read_utf(cursor: &mut Cursor<Vec<u8>>) -> Result<String, Box<dyn std::error::Error>> {
    let len = cursor.get_u16() as usize;
    let mut bytes = vec![0u8; len];
    cursor.copy_to_slice(&mut bytes);
    Ok(String::from_utf8(bytes)?)
}

fn read_nullable_text(cursor: &mut Cursor<Vec<u8>>) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let present = cursor.get_u8() != 0;
    if present {
        Ok(Some(read_utf(cursor)?))
    } else {
        Ok(None)
    }
}

fn write_utf(buf: &mut BytesMut, s: &str) {
    let bytes = s.as_bytes();
    buf.put_u16(bytes.len() as u16);
    buf.put_slice(bytes);
}

fn write_nullable_text(buf: &mut BytesMut, s: Option<&str>) {
    match s {
        Some(text) => {
            buf.put_u8(1);
            write_utf(buf, text);
        },
        None => buf.put_u8(0),
    }
}
