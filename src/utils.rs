use serde_json::{Map, Value};
use std::collections::HashMap;
use std::{fs, str::Utf8Error};
use warp::hyper::{Body, Response, StatusCode};
use warp::Reply;

#[derive(Clone, Debug)]
pub struct Calendar {
    pub name: String,
    pub url: String,
    pub path: String,
    pub color: Option<String>,
}

pub fn parse_calendar_json(path: &str) -> HashMap<String, Calendar> {
    let json_data = fs::read_to_string(path).expect("Unable to read calendars file");
    let cals: serde_json::Value =
        serde_json::from_str(&json_data).expect("The calendars file is not a valid json file");
    let cals = &cals["calendars"];
    let cals: &Map<String, Value> = cals
        .as_object()
        .expect("The json file doesn't follow the required format");
    let mut calendars = HashMap::new();
    for (url, value) in cals {
        let url_vec: Vec<&str> = url.split("/").collect();
        calendars.insert(
            String::from(url_vec[url_vec.len() - 1]),
            Calendar {
                name: String::from(value["name"].as_str().unwrap_or("Unamed calendar")),
                url: String::from(url),
                path: String::from(url_vec[url_vec.len() - 1]),
                color: value
                    .get("color")
                    .map(|v| String::from(v.as_str().unwrap())),
            },
        );
    }
    calendars
}

#[derive(Debug)]
pub enum LinkalError {
    XMLError(exile::error::Error),
    ParsingError(Utf8Error),
    UpstreamError(ureq::Error),
    IOError(std::io::Error),
}

impl From<Utf8Error> for LinkalError {
    fn from(err: Utf8Error) -> Self {
        LinkalError::ParsingError(err)
    }
}

impl From<ureq::Error> for LinkalError {
    fn from(err: ureq::Error) -> Self {
        LinkalError::UpstreamError(err)
    }
}

impl From<std::io::Error> for LinkalError {
    fn from(err: std::io::Error) -> Self {
        LinkalError::IOError(err)
    }
}

impl From<exile::error::Error> for LinkalError {
    fn from(err: exile::error::Error) -> Self {
        LinkalError::XMLError(err)
    }
}

// Warp error handling and propagation
// Courtesy of https://github.com/seanmonstar/warp/pull/909#issuecomment-1184854848
impl LinkalError {
    fn status_code_body(self: LinkalError) -> (StatusCode, String) {
        match self {
            LinkalError::ParsingError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            LinkalError::UpstreamError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            LinkalError::IOError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()), // Mismatched hash
            LinkalError::XMLError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }
}

impl Reply for LinkalError {
    fn into_response(self) -> Response<Body> {
        let (status, body) = self.status_code_body();
        Response::builder()
            .status(status)
            .body(body.into())
            .expect("Could not construct Response")
    }
}

pub fn into_response<S: Reply, E: Reply>(reply_res: Result<S, E>) -> Response<Body> {
    match reply_res {
        Ok(resp) => resp.into_response(),
        Err(err) => err.into_response(),
    }
}
