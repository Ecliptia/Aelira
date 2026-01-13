use std::path::Path;
use tokio::fs::File;
use tokio_util::codec::FramedRead;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use crate::playback::demuxers::webm::WebmOpusDemuxer;
use crate::utils::encoding::{DecodedTrack, DecodedInfo, encode_track};
use crate::managers::sources::Source;
use crate::models::load_tracks::{LoadTracksResponse, LoadType, LoadResultData};
use async_trait::async_trait;

pub struct LocalSource;

#[async_trait]
impl Source for LocalSource {
    fn name(&self) -> &'static str {
        "local"
    }

    fn priority(&self) -> u32 {
        20
    }

    fn search_terms(&self) -> Vec<&'static str> {
        vec!["local", "file"]
    }

    fn patterns(&self) -> Vec<&'static str> {
        vec![r"^(local|file):"]
    }

    async fn search(&self, query: &str, _search_type: &str) -> LoadTracksResponse {
        self.resolve(query).await
    }

    async fn resolve(&self, url: &str) -> LoadTracksResponse {
        let clean_path = if url.starts_with("local:") {
            &url[6..]
        } else if url.starts_with("file:") {
            &url[5..]
        } else {
            url
        };
        
        let path = Path::new(clean_path);
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return LoadTracksResponse {
                load_type: LoadType::Empty,
                data: LoadResultData::Empty(serde_json::json!({})),
            }
        };

        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probed = match symphonia::default::get_probe()
            .format(&hint, mss, &Default::default(), &Default::default()) {
                Ok(p) => p,
                Err(_) => return LoadTracksResponse {
                    load_type: LoadType::Empty,
                    data: LoadResultData::Empty(serde_json::json!({})),
                }
            };

        let format = probed.format;
        let duration = format.tracks().iter()
            .filter_map(|t| t.codec_params.time_base.zip(t.codec_params.n_frames))
            .map(|(tb, frames)| tb.calc_time(frames).seconds * 1000)
            .next()
            .unwrap_or(0);

        let info = DecodedInfo {
            title: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            author: "unknown".to_string(),
            length: duration,
            identifier: clean_path.to_string(),
            is_stream: false,
            uri: Some(clean_path.to_string()),
            artwork_url: None,
            isrc: None,
            source_name: "local".to_string(),
            position: 0,
        };

        let track = DecodedTrack {
            encoded: encode_track(&info),
            info,
            plugin_info: serde_json::json!({}),
            user_data: serde_json::json!({}),
        };

        LoadTracksResponse {
            load_type: LoadType::Track,
            data: LoadResultData::Track(track),
        }
    }

    async fn load_stream(&self, identifier: &str) -> Option<FramedRead<File, WebmOpusDemuxer>> {
        let clean_path = if identifier.starts_with("local:") {
            &identifier[6..]
        } else if identifier.starts_with("file:") {
            &identifier[5..]
        } else {
            identifier
        };
        let file = File::open(clean_path).await.ok()?;
        Some(FramedRead::new(file, WebmOpusDemuxer::new()))
    }
}