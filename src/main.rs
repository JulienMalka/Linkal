mod filters;
mod handlers;
mod propfind;
mod utils;
use utils::Calendar;

#[tokio::main]
async fn main() {
    //TODO Parse command line arguments to get this path
    let path = "./data/calendars.json";
    let calendars = utils::parse_calendar_json(path);
    pretty_env_logger::init();
    let routes = filters::api(calendars);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
