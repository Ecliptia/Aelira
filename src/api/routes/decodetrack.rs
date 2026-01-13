use warp::{Filter, Reply};
use crate::aelira::AeliraRef;
use crate::utils::encoding::decode_track;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct DecodeTrackQuery {
    #[serde(rename = "encodedTrack")]
    pub encoded_track: String,
}

pub fn handler(aelira: AeliraRef) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let with_aelira = warp::any().map(move || aelira.clone());

    warp::path("v4")
        .and(warp::path("decodetrack"))
        .and(warp::get())
        .and(warp::query::<DecodeTrackQuery>())
        .and(with_aelira)
        .map(|query: DecodeTrackQuery, _aelira: AeliraRef| {
            let encoded = query.encoded_track.replace(' ', "+");
            
            match decode_track(&encoded) {
                Ok(decoded) => warp::reply::json(&decoded).into_response(),
                Err(e) => {
                    let error = json!({
                        "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        "status": 400,
                        "error": "Bad Request",
                        "message": format!("Failed to decode track: {}", e),
                        "path": "/v4/decodetrack"
                    });
                    warp::reply::with_status(warp::reply::json(&error), warp::http::StatusCode::BAD_REQUEST).into_response()
                }
            }
        })
}
