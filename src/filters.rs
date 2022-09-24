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
    warp::path::end()
        .and(warp::body::bytes())
        .and_then(handlers::handle_index)
}

pub fn get_home_url() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("principals" / "mock")
        .and(warp::body::bytes())
        .and_then(handlers::handle_home_url)
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
        .and_then(handlers::handle_cals)
}

pub fn get_calendars_proppatch(
    calendars: HashMap<String, Calendar>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("cals")
        .and(warp::body::bytes())
        .and(warp::method())
        .and(with_cals(calendars))
        .and_then(handlers::handle_cals_proppatch)
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
        .and_then(handlers::handle_events)
}

pub fn api(
    calendars: HashMap<String, Calendar>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    options_request_cals()
        .or(options_request())
        .or(index())
        .or(get_home_url())
        .or(get_calendars(calendars.clone()))
        .or(get_calendars_proppatch(calendars.clone()))
        .or(get_events(calendars.clone()))
        .or(well_known())
}
