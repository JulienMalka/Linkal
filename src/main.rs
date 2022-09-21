use serde_json;
use serde_json::{Map, Value};
use std::fs;
use warp::reject::Reject;
mod filters;
mod handlers;
use std::collections::HashMap;

#[derive(Debug)]
struct ConversionError;
impl Reject for ConversionError {}

#[derive(Clone, Debug)]
pub struct Calendar {
    name: String,
    url: String,
    path: String,
}

fn parse_calendar_json(path: &str) -> HashMap<String, Calendar> {
    let json_data = fs::read_to_string(path).expect("Unable to read calendars.json file");
    let cals: serde_json::Value = serde_json::from_str(&json_data).unwrap();
    let cals = cals["calendars"];
    let cals: Map<String, Value> = cals.as_object().unwrap().clone();
    let mut calendars = HashMap::new();
    for (k, v) in cals {
        let split = k.clone();
        let splitted = split.split("/");
        let vec: Vec<&str> = splitted.collect();
        calendars.insert(
            String::from(vec[vec.len() - 1]),
            Calendar {
                name: String::from(v["name"].as_str().unwrap_or("Unamed calendar")),
                url: String::from(k),
                path: String::from(vec[vec.len() - 1]),
            },
        );
    }
    calendars
}

#[tokio::main]
async fn main() {
    //TODO Parse command line arguments to get this path
    let path = "./data/calendars.json";
    let calendars = parse_calendar_json(path);
    pretty_env_logger::init();
    let routes = filters::api(calendars);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
