// Warp error handling and propagation
// Courtesy of https://github.com/seanmonstar/warp/pull/909#issuecomment-1184854848

use std::str::Utf8Error;
use warp::hyper::{Body, Response, StatusCode};
use warp::Reply;

#[derive(Debug)]
pub enum LinkalError {
    Forbidden,
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

impl LinkalError {
    fn status_code_body(self: LinkalError) -> (StatusCode, String) {
        match self {
            LinkalError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN".to_string()),
            // TODO: figure out which DF errors to propagate, we have ones that are the server's fault
            // here too (e.g. ResourcesExhaused) and potentially some that leak internal information
            // (e.g. ObjectStore?)
            LinkalError::ParsingError(_) => (StatusCode::BAD_REQUEST, "PARSING".to_string()),
            LinkalError::UpstreamError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            LinkalError::IOError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()), // Mismatched hash
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
