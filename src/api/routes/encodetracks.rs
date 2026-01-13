use warp::{Filter, Reply};
use crate::aelira::AeliraRef;
use crate::utils::encoding::{encode_track, DecodedInfo};

pub fn handler(aelira: AeliraRef) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let with_aelira = warp::any().map(move || aelira.clone());

    warp::path("v4")
        .and(warp::path("encodetracks"))
        .and(warp::post())
        .and(warp::body::json::<Vec<DecodedInfo>>())
        .and(with_aelira)
        .map(|tracks: Vec<DecodedInfo>, _aelira: AeliraRef| {
            let mut encoded_tracks = Vec::new();
            
            for info in tracks {
                let encoded = encode_track(&info);
                encoded_tracks.push(encoded);
            }
            
            warp::reply::json(&encoded_tracks).into_response()
        })
}
