use warp::{Filter, Reply};
use crate::aelira::AeliraRef;
use crate::utils::encoding::{encode_track, DecodedInfo};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct EncodeTrackQuery {
    pub track: String,
}

pub fn handler(aelira: AeliraRef) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let with_aelira = warp::any().map(move || aelira.clone());

    warp::path("v4")
        .and(warp::path("encodetrack"))
        .and(warp::get())
        .and(warp::query::<EncodeTrackQuery>())
        .and(with_aelira)
        .map(|query: EncodeTrackQuery, _aelira: AeliraRef| {
            match serde_json::from_str::<DecodedInfo>(&query.track) {
                Ok(info) => {
                    let encoded = encode_track(&info);
                    warp::reply::json(&encoded).into_response()
                },
                Err(e) => {
                    let error = json!({
                        "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                        "status": 400,
                        "error": "Bad Request",
                        "message": format!("Failed to parse track info: {}", e),
                        "path": "/v4/encodetrack"
                    });
                    warp::reply::with_status(warp::reply::json(&error), warp::http::StatusCode::BAD_REQUEST).into_response()
                }
            }
        })
}
