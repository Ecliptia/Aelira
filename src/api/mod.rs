pub mod routes;
pub mod middlewares;

use warp::http::StatusCode;

pub async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, std::convert::Infallible> {
    if err.is_not_found() {
        return Ok(warp::reply::with_status("Not Found", StatusCode::NOT_FOUND));
    }

    if let Some(_) = err.find::<middlewares::auth::AuthError>() {
        return Ok(warp::reply::with_status("Unauthorized", StatusCode::UNAUTHORIZED));
    }

    eprintln!("Unhandled rejection: {:?}", err);
    Ok(warp::reply::with_status("Internal Server Error", StatusCode::INTERNAL_SERVER_ERROR))
}