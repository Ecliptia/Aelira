use warp::Filter;
use warp::reject::Reject;

#[derive(Debug)]
pub struct AuthError;

impl Reject for AuthError {}

pub fn with_auth(password: Option<String>) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .map(move |auth_header: Option<String>| {
            if password.is_none() {
                return true;
            }
            
            if let Some(ref pass) = password {
                if let Some(header) = auth_header {
                    return header == *pass;
                }
            }
            false
        })
        .and_then(|is_valid: bool| async move {
            if is_valid {
                Ok(())
            } else {
                Err(warp::reject::custom(AuthError))
            }
        })
        .untuple_one() 
}