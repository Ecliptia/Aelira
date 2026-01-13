use warp::Filter;
use crate::aelira::AeliraRef;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoadTracksQuery {
    pub identifier: String,
}

pub fn handler(aelira: AeliraRef) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let with_aelira = warp::any().map(move || aelira.clone());

    warp::path("v4")
        .and(warp::path("loadtracks"))
        .and(warp::get())
        .and(warp::query::<LoadTracksQuery>())
        .and(with_aelira)
        .and_then(|query: LoadTracksQuery, aelira: AeliraRef| async move {
            aelira.stats.increment_api_request("/v4/loadtracks");
            let response = aelira.sources.load_tracks(&query.identifier).await;
            Ok::<_, warp::Rejection>(warp::reply::json(&response))
        })
}
