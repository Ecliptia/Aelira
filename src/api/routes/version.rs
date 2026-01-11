use crate::aelira::AeliraRef;
use warp::Filter;

pub fn version(aelira: AeliraRef) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("version").and(warp::get()).map(move || aelira.version.clone())
}