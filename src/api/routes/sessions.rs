use crate::aelira::AeliraRef;
use crate::managers::players::{Player, TrackData, VoiceState};
use crate::utils::encoding::decode_track;
use crate::utils::{log, Level};
use serde::Deserialize;
use warp::http::StatusCode;
use warp::{Filter, Reply};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUpdatePayload {
    pub resuming: Option<bool>,
    pub timeout: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct PlayerUpdatePayload {
    pub track: Option<UpdatePlayerTrack>,
    pub encoded_track: Option<String>,
    pub _position: Option<i64>,
    pub _end_time: Option<i64>,
    pub volume: Option<u16>,
    pub paused: Option<bool>,
    pub voice: Option<VoiceState>,
    pub _filters: Option<serde_json::Value>,
    pub _no_replace: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct UpdatePlayerTrack {
    pub encoded: Option<String>,
    pub identifier: Option<String>,
    pub _user_data: Option<serde_json::Value>,
}

pub fn handler(aelira: AeliraRef) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let with_aelira = warp::any().map(move || aelira.clone());

    let base_sessions = warp::path("v4")
        .and(warp::path("sessions"))
        .and(warp::path::param::<String>());

    let update_session = base_sessions.clone()
        .and(warp::path::end())
        .and(warp::patch())
        .and(warp::body::json())
        .and(with_aelira.clone())
        .map(|session_id: String, body: SessionUpdatePayload, aelira: AeliraRef| {
            let manager = aelira.sessions.lock().unwrap();

            if manager.sessions.contains_key(&session_id) {
                let response = serde_json::json!({
                    "resuming": body.resuming.unwrap_or(false),
                    "timeout": body.timeout.unwrap_or(60)
                });
                return warp::reply::json(&response).into_response();
            }
            warp::reply::with_status("Session not found", StatusCode::NOT_FOUND).into_response()
        });

    let base_players = base_sessions.clone()
        .and(warp::path("players"));

    let get_players = base_players.clone()
        .and(warp::path::end())
        .and(warp::get())
        .and(with_aelira.clone())
        .map(|session_id: String, aelira: AeliraRef| {
            let manager = aelira.sessions.lock().unwrap();
            if let Some(session) = manager.sessions.get(&session_id) {
                let players = session.players.lock().unwrap();
                let player_list: Vec<&Player> = players.players.values().collect();
                return warp::reply::json(&player_list).into_response();
            }
            warp::reply::with_status("Session not found", StatusCode::NOT_FOUND).into_response()
        });

    let player_by_id = base_players.clone()
        .and(warp::path::param::<String>());

    let get_player = player_by_id.clone()
        .and(warp::path::end())
        .and(warp::get())
        .and(with_aelira.clone())
        .map(|session_id: String, guild_id: String, aelira: AeliraRef| {
            let manager = aelira.sessions.lock().unwrap();
            if let Some(session) = manager.sessions.get(&session_id) {
                let mut players = session.players.lock().unwrap();
                let player = players.get_or_create(guild_id);
                return warp::reply::json(&player).into_response();
            }
            warp::reply::with_status("Session not found", StatusCode::NOT_FOUND).into_response()
        });

    let patch_player = player_by_id.clone()
        .and(warp::path::end())
        .and(warp::patch())
        .and(warp::body::json())
        .and(with_aelira.clone())
        .and_then(|session_id: String, guild_id: String, body: PlayerUpdatePayload, aelira: AeliraRef| async move {
            log(Level::Debug, "API", format!("PATCH Player Session: {}, Guild: {}", session_id, guild_id));

            let mut identifier_to_resolve = None;

            {
                let manager = aelira.sessions.lock().unwrap();
                if let Some(session) = manager.sessions.get(&session_id) {
                    let mut players = session.players.lock().unwrap();
                    let player = players.get_or_create(guild_id.clone());

                    if let Some(voice) = &body.voice {
                        let should_connect = match &player.voice {
                            Some(current) => current.token != voice.token || current.endpoint != voice.endpoint || current.session_id != voice.session_id,
                            None => true
                        };

                        if should_connect {
                             player.connect(voice.clone(), session.user_id.clone());
                        }
                    }

                    if let Some(paused) = body.paused { player.paused = paused; }
                    if let Some(vol) = body.volume { player.volume = vol; }

                    if let Some(track_upd) = &body.track {
                        if let Some(encoded) = &track_upd.encoded {
                             if let Ok(decoded) = decode_track(encoded) {
                                 player.track = Some(TrackData { encoded: encoded.clone(), info: decoded.info });
                                 player.play();
                             }
                        } else {
                            identifier_to_resolve = track_upd.identifier.clone();
                        }
                    } else if let Some(encoded) = &body.encoded_track {
                        if let Ok(decoded) = decode_track(encoded) {
                            player.track = Some(TrackData { encoded: encoded.clone(), info: decoded.info });
                            player.play();
                        }
                    }
                } else {
                    return Ok::<_, warp::Rejection>(warp::reply::with_status("Session not found", StatusCode::NOT_FOUND).into_response());
                }
            }

            if let Some(identifier) = identifier_to_resolve {
                let res = aelira.sources.load_tracks(&identifier).await;
                if let crate::models::load_tracks::LoadResultData::Track(track) = res.data {
                    let manager = aelira.sessions.lock().unwrap();
                    if let Some(session) = manager.sessions.get(&session_id) {
                        let mut players = session.players.lock().unwrap();
                        if let Some(player) = players.players.get_mut(&guild_id) {
                            player.track = Some(TrackData { encoded: track.encoded, info: track.info });
                            player.play();
                        }
                    }
                } else {
                     return Ok(warp::reply::with_status("Track resolution failed", StatusCode::BAD_REQUEST).into_response());
                }
            }

            let manager = aelira.sessions.lock().unwrap();
            if let Some(session) = manager.sessions.get(&session_id) {
                let players = session.players.lock().unwrap();
                 if let Some(player) = players.players.get(&guild_id) {
                     return Ok(warp::reply::json(&player).into_response());
                 }
            }

            Ok(warp::reply::with_status("Session not found", StatusCode::NOT_FOUND).into_response())
        });

    let delete_player = player_by_id.clone()
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_aelira.clone())
        .map(|session_id: String, guild_id: String, aelira: AeliraRef| {
            let manager = aelira.sessions.lock().unwrap();
            if let Some(session) = manager.sessions.get(&session_id) {
                let mut players = session.players.lock().unwrap();
                if players.players.remove(&guild_id).is_some() {
                    return StatusCode::NO_CONTENT.into_response();
                }
                return warp::reply::with_status("Player not found", StatusCode::NOT_FOUND).into_response();
            }
            warp::reply::with_status("Session not found", StatusCode::NOT_FOUND).into_response()
        });

    update_session
        .or(get_players)
        .or(get_player)
        .or(patch_player)
        .or(delete_player)
}
