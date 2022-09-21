use crate::handlers;
use crate::Calendar;
use std::collections::HashMap;
use warp::Filter;

fn with_cals(
    calendars: HashMap<String, Calendar>,
) -> impl Filter<Extract = (HashMap<String, Calendar>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || calendars.clone())
}

pub fn index() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path::end().and_then(handlers::handle_index)
}

pub fn get_home_url() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("principals" / "mock").and_then(handlers::handle_home_url)
}

pub fn get_calendars(
    calendars: HashMap<String, Calendar>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("cals")
        .and(warp::body::bytes())
        .and(with_cals(calendars))
        .and_then(handlers::handle_cals)
}

pub fn get_events(
    calendars: HashMap<String, Calendar>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("cals" / String)
        .and(with_cals(calendars))
        .and_then(handlers::handle_events)
}

pub fn api(
    calendars: HashMap<String, Calendar>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    index()
        .or(get_home_url())
        .or(get_calendars(calendars.clone()))
        .or(get_events(calendars.clone()))
}
