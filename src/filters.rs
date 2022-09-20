use crate::handlers;
use crate::Calendar;
use warp::Filter;

fn with_cals(
    calendars: Vec<Calendar>,
) -> impl Filter<Extract = (Vec<Calendar>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || calendars.clone())
}

pub fn index() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path::end().and_then(handlers::handle_index)
}

pub fn get_home_url() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("principals" / "ens").and_then(handlers::handle_home_url)
}

pub fn get_calendars(
    calendars: Vec<Calendar>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("cals")
        .and(with_cals(calendars))
        .and_then(handlers::handle_cals)
}

pub fn get_events() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("cals" / "personal").and_then(handlers::handle_events)
}

pub fn get_events_inutile(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("cals" / "inutile").and_then(handlers::handle_events_inutile)
}

pub fn api(
    calendars: Vec<Calendar>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    index()
        .or(get_home_url())
        .or(get_calendars(calendars.clone()))
        .or(get_events())
        .or(get_events_inutile())
}
