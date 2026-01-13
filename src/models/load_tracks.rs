use serde::Serialize;
use crate::utils::encoding::DecodedTrack;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadTracksResponse {
    pub load_type: LoadType,
    pub data: LoadResultData,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum LoadType {
    Track,
    Playlist,
    Search,
    Empty,
    Error,
}

#[derive(Serialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum LoadResultData {
    Track(DecodedTrack),
    Playlist(PlaylistData),
    Search(Vec<DecodedTrack>),
    Empty(serde_json::Value),
    Error(ErrorData),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistData {
    pub info: PlaylistInfo,
    pub plugin_info: serde_json::Value,
    pub tracks: Vec<DecodedTrack>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistInfo {
    pub name: String,
    pub selected_track: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorData {
    pub message: String,
    pub severity: String,
    pub cause: String,
}
