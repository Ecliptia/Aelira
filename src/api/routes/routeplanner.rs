use warp::{Filter, Reply};
use crate::aelira::AeliraRef;
use serde::Deserialize;
use warp::http::StatusCode;

#[derive(Deserialize)]
pub struct FreeAddressPayload {
    pub address: String,
}

pub fn handler(aelira: AeliraRef) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let with_aelira = warp::any().map(move || aelira.clone());

    let status = warp::path("v4")
        .and(warp::path("routeplanner"))
        .and(warp::path("status"))
        .and(warp::get())
        .and(with_aelira.clone())
        .map(|aelira: AeliraRef| {
            let status = aelira.route_planner.get_status();
            if status.class.is_none() {
                warp::reply::with_status(warp::reply::reply(), StatusCode::NO_CONTENT).into_response()
            } else {
                warp::reply::json(&status).into_response()
            }
        });

    let free_address = warp::path("v4")
        .and(warp::path("routeplanner"))
        .and(warp::path("free"))
        .and(warp::path("address"))
        .and(warp::post())
        .and(warp::body::json::<FreeAddressPayload>())
        .and(with_aelira.clone())
        .map(|body: FreeAddressPayload, aelira: AeliraRef| {
            aelira.route_planner.unmark_address(&body.address);
            StatusCode::NO_CONTENT
        });

    let free_all = warp::path("v4")
        .and(warp::path("routeplanner"))
        .and(warp::path("free"))
        .and(warp::path("all"))
        .and(warp::post())
        .and(with_aelira)
        .map(|aelira: AeliraRef| {
            aelira.route_planner.unmark_all_addresses();
            StatusCode::NO_CONTENT
        });

    status.or(free_address).or(free_all)
}
