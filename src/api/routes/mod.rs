use warp::Filter;
use crate::aelira::AeliraRef;
use crate::api::middlewares::auth::with_auth;

mod version;
mod stats;

pub fn all_routes(aelira: AeliraRef) -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let auth = with_auth(aelira.password.clone());

    let version_route = version::version(aelira.clone());

    let v4_stats = warp::path("v4")
        .and(auth.clone())
        .and(warp::path("stats"))
        .and(stats::handler(aelira.clone()));

    version_route.or(v4_stats)
}