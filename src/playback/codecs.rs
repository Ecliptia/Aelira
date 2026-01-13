use symphonia::core::probe::Hint;

#[allow(dead_code)]
pub enum AudioContainer {
    Webm,
    Mp4,
    Ogg,
    Wav,
    Mp3,
    Flac,
    Aac,
}

pub fn map_mime_to_hint(mime: &str) -> Hint {
    let mut hint = Hint::new();
    match mime {
        "audio/webm" | "video/webm" => { hint.with_extension("webm"); },
        "audio/mp4" | "video/mp4" => { hint.with_extension("mp4"); },
        "audio/ogg" | "application/ogg" => { hint.with_extension("ogg"); },
        "audio/wav" | "audio/x-wav" => { hint.with_extension("wav"); },
        "audio/mpeg" | "audio/mp3" => { hint.with_extension("mp3"); },
        "audio/flac" | "audio/x-flac" => { hint.with_extension("flac"); },
        "audio/aac" | "audio/aacp" => { hint.with_extension("aac"); },
        _ => {}
    }
    hint
}
