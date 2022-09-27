mod filters;
mod handlers;
mod propfind;
mod utils;
use clap::Parser;
use std::path::PathBuf;
use utils::Calendar;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, value_parser, value_name = "FILE")]
    calendar_file: PathBuf,
}

#[tokio::main]
async fn main() {
    //TODO Parse command line arguments to get this path
    let cli = Cli::parse();

    let path = cli.calendar_file;
    let calendars = utils::parse_calendar_json(path.to_str().unwrap());
    let routes = filters::api(calendars);
    warp::serve(routes).run(([127, 0, 0, 1], 4145)).await;
}
