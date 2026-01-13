use std::collections::HashMap;
use async_trait::async_trait;
use crate::playback::demuxers::webm::WebmOpusDemuxer;
use crate::models::load_tracks::{LoadTracksResponse, LoadType, LoadResultData};
use tokio::fs::File;
use tokio_util::codec::FramedRead;
use regex::Regex;

#[async_trait]
pub trait Source: Send + Sync {
    fn name(&self) -> &'static str;
    fn priority(&self) -> u32 { 10 }
    fn search_terms(&self) -> Vec<&'static str> { Vec::new() }
    fn patterns(&self) -> Vec<&'static str> { Vec::new() }
    fn _matches(&self, query: &str) -> bool {
        for pattern in self.patterns() {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(query) {
                    return true;
                }
            }
        }
        false
    }
    async fn search(&self, query: &str, search_type: &str) -> LoadTracksResponse;
    async fn resolve(&self, url: &str) -> LoadTracksResponse;
    async fn load_stream(&self, identifier: &str) -> Option<FramedRead<File, WebmOpusDemuxer>>;
}

struct SourcePattern {
    regex: Regex,
    source_name: String,
    priority: u32,
}

pub struct SourceManager {
    sources: HashMap<String, Box<dyn Source>>,
    search_term_map: HashMap<String, String>,
    patterns: Vec<SourcePattern>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self { 
            sources: HashMap::new(),
            search_term_map: HashMap::new(),
            patterns: Vec::new(),
        }
    }

    pub fn register(&mut self, source: Box<dyn Source>) {
        let name = source.name().to_string();
        let priority = source.priority();
        
        for term in source.search_terms() {
            self.search_term_map.insert(term.to_string(), name.clone());
        }

        for pattern in source.patterns() {
            if let Ok(re) = Regex::new(pattern) {
                self.patterns.push(SourcePattern {
                    regex: re,
                    source_name: name.clone(),
                    priority,
                });
            }
        }
        
        self.patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.sources.insert(name, source);
    }

    pub async fn load_tracks(&self, identifier: &str) -> LoadTracksResponse {
        if std::path::Path::new(identifier).exists() {
            if let Some(source) = self.sources.get("local") {
                let res = source.resolve(identifier).await;
                if !matches!(res.load_type, LoadType::Empty) {
                    return res;
                }
            }
        }

        for pattern in &self.patterns {
            if pattern.regex.is_match(identifier) {
                if let Some(source) = self.sources.get(&pattern.source_name) {
                    let res = source.resolve(identifier).await;
                    if !matches!(res.load_type, LoadType::Empty) {
                        return res;
                    }
                }
            }
        }

        if let Some((prefix, query)) = identifier.split_once(':') {
            if prefix.len() > 1 {
                if let Some(source_name) = self.search_term_map.get(prefix) {
                    if let Some(source) = self.sources.get(source_name) {
                        return source.search(query, "track").await;
                    }
                }
            }
        }

        let results = self.unified_search(identifier).await;
        if results.is_empty() {
            return LoadTracksResponse {
                load_type: LoadType::Empty,
                data: LoadResultData::Empty(serde_json::json!({})),
            };
        }

        LoadTracksResponse {
            load_type: LoadType::Search,
            data: LoadResultData::Search(results),
        }
    }

    pub async fn unified_search(&self, query: &str) -> Vec<crate::utils::encoding::DecodedTrack> {
        let mut results = Vec::new();
        for source in self.sources.values() {
            let res = source.search(query, "track").await;
            if let LoadResultData::Search(tracks) = res.data {
                results.extend(tracks);
            } else if let LoadResultData::Track(track) = res.data {
                results.push(track);
            }
        }
        results
    }

    #[allow(dead_code)]
    pub async fn load_stream(&self, identifier: &str) -> Option<FramedRead<File, WebmOpusDemuxer>> {
        for pattern in &self.patterns {
            if pattern.regex.is_match(identifier) {
                if let Some(source) = self.sources.get(&pattern.source_name) {
                    return source.load_stream(identifier).await;
                }
            }
        }

        if let Some((prefix, _)) = identifier.split_once(':') {
            if let Some(source_name) = self.search_term_map.get(prefix) {
                if let Some(source) = self.sources.get(source_name) {
                    return source.load_stream(identifier).await;
                }
            }
        }

        None
    }
    
    pub fn _list(&self) -> Vec<String> {
        self.sources.keys().cloned().collect()
    }
}
