use warp::Filter;
use crate::aelira::AeliraRef;
use crate::api::middlewares::auth::with_auth;

mod websocket;
mod version;
mod stats;
mod sessions;
mod loadtracks;
mod info;
mod decodetrack;
mod decodetracks;
mod encodetrack;
mod encodetracks;
mod routeplanner;

pub fn all_routes(aelira: AeliraRef) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let auth = with_auth(aelira.password.clone());

    let version_route = version::version(aelira.clone());
    let websocket_route = websocket::handler(aelira.clone());
    
    let sessions_route = sessions::handler(aelira.clone());
    let loadtracks_route = loadtracks::handler(aelira.clone());
    let info_route = info::handler(aelira.clone());
    let decodetrack_route = decodetrack::handler(aelira.clone());
    let decodetracks_route = decodetracks::handler(aelira.clone());
    let encodetrack_route = encodetrack::handler(aelira.clone());
    let encodetracks_route = encodetracks::handler(aelira.clone());
    let routeplanner_route = routeplanner::handler(aelira.clone());

    let v4_stats = warp::path("v4")
        .and(auth.clone())
        .and(warp::path("stats"))
        .and(stats::handler(aelira.clone()));

    version_route
        .or(websocket_route)
        .or(v4_stats)
        .or(sessions_route)
        .or(loadtracks_route)
        .or(info_route)
        .or(decodetrack_route)
        .or(decodetracks_route)
        .or(encodetrack_route)
        .or(encodetracks_route)
        .or(routeplanner_route)
}
