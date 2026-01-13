use warp::{Filter, Reply};
use crate::aelira::AeliraRef;
use crate::utils::encoding::decode_track;
use serde_json::json;

pub fn handler(aelira: AeliraRef) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let with_aelira = warp::any().map(move || aelira.clone());

    warp::path("v4")
        .and(warp::path("decodetracks"))
        .and(warp::post())
        .and(warp::body::json::<Vec<String>>())
        .and(with_aelira)
        .map(|tracks: Vec<String>, _aelira: AeliraRef| {
            let mut decoded_tracks = Vec::new();
            
            for encoded in tracks {
                let encoded = encoded.replace(' ', "+");
                match decode_track(&encoded) {
                    Ok(decoded) => decoded_tracks.push(decoded),
                    Err(e) => {
                        let error = json!({
                            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                            "status": 400,
                            "error": "Bad Request",
                            "message": format!("Failed to decode track: {}", e),
                            "path": "/v4/decodetracks"
                        });
                        return warp::reply::with_status(warp::reply::json(&error), warp::http::StatusCode::BAD_REQUEST).into_response();
                    }
                }
            }
            
            warp::reply::json(&decoded_tracks).into_response()
        })
}
