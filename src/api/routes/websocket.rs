use warp::{Filter, Reply};
use crate::aelira::AeliraRef;
use warp::http::StatusCode;

pub fn handler(aelira: AeliraRef) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let with_aelira = warp::any().map(move || aelira.clone());

    warp::path("v4")
        .and(warp::path("websocket"))
        .and(warp::ws())
        .and(warp::header::header("authorization"))
        .and(warp::header::header("user-id"))
        .and(warp::header::header("client-name"))
        .and(warp::header::optional::<String>("session-id"))
        .and(with_aelira)
        .map(|ws: warp::ws::Ws, auth: String, user_id: String, client_name: String, session_id: Option<String>, aelira: AeliraRef| {
            if let Some(pass) = &aelira.password {
                if *pass != auth {
                    return warp::reply::with_status("Unauthorized", StatusCode::UNAUTHORIZED).into_response();
                }
            }

            if !user_id.chars().all(char::is_numeric) {
                 return warp::reply::with_status("Invalid User ID", StatusCode::BAD_REQUEST).into_response();
            }

                        ws.on_upgrade(move |socket| async move {

                            crate::socket::handle_socket(socket, client_name, user_id, session_id, aelira).await;

                        }).into_response()

            
        })
}