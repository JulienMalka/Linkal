use crate::{handlers, utils::into_response, Calendar};
use std::collections::HashMap;
use warp::hyper::{Method, StatusCode};
use warp::{reject, Filter, Rejection, Reply};

#[derive(Debug)]
struct MethodError;
impl reject::Reject for MethodError {}

const PROPFIND_METHOD: &'static str = "PROPFIND";
const REPORT_METHOD: &'static str = "REPORT";
const PROPPATCH_METHOD: &'static str = "PROPPATCH";

pub fn get_req() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().map(warp::reply)
}

fn method(name: &'static str) -> impl Filter<Extract = (), Error = Rejection> + Clone + Sized {
    warp::method()
        .and_then(move |m: Method| async move {
            if m == name {
                Ok(())
            } else {
                Err(reject::custom(MethodError))
            }
        })
        .untuple_one()
}

pub async fn handle_not_found(reject: Rejection) -> Result<impl Reply, Rejection> {
    if reject.is_not_found() {
        Ok(StatusCode::NOT_FOUND)
    } else {
        Err(reject)
    }
}

pub fn with_cals(
    calendars: HashMap<String, Calendar>,
) -> impl Filter<Extract = (HashMap<String, Calendar>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || calendars.clone())
}

pub fn index() -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    warp::path::end()
        .and(warp::body::bytes())
        .then(move |b: bytes::Bytes| handlers::handle_propfind_locally(b, "/"))
        .map(into_response)
}

pub fn allowed_method() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::options()
        .or(method(PROPFIND_METHOD))
        .unify()
        .or(method(REPORT_METHOD))
        .unify()
}

pub fn get_home_url() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("principals" / "mock")
        .and(warp::body::bytes())
        .then(move |b: bytes::Bytes| handlers::handle_propfind_locally(b, "/principals/mock/"))
        .map(into_response)
}

pub fn options_request() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::options().and_then(handlers::handle_options)
}

pub fn options_request_cals(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::options()
        .and(warp::path("cals"))
        .and_then(handlers::handle_options_cals)
}

pub fn get_calendars(
    calendars: HashMap<String, Calendar>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("cals")
        .and(warp::method())
        .and(with_cals(calendars))
        .and(warp::header::<u32>("Depth"))
        .and(warp::body::bytes())
        .then(handlers::handle_cals)
        .map(into_response)
}

pub fn well_known() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(".well-known" / "caldav").and_then(handlers::handle_well_known)
}

pub fn get_events(
    calendars: HashMap<String, Calendar>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("cals" / String)
        .and(warp::body::bytes())
        .and(warp::method())
        .and(with_cals(calendars))
        .and(warp::header::optional::<u32>("Depth"))
        .then(handlers::handle_events)
        .map(into_response)
}

pub fn api(
    calendars: HashMap<String, Calendar>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    allowed_method().and(
        options_request_cals()
            .or(options_request())
            .or(index())
            .or(get_home_url())
            .or(get_calendars(calendars.clone()))
            .or(get_events(calendars.clone()))
            .or(well_known()),
    )
}
