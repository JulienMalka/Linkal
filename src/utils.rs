// Warp error handling and propagation
// Courtesy of https://github.com/seanmonstar/warp/pull/909#issuecomment-1184854848
//
// Usage:
//
//   1) A handler function, instead of returning a Warp reply/rejection, returns a
//   `Result<Reply, ApiError>.`
//
//   This is because rejections are meant to say "this filter can't handle this request, but maybe
//   some other can" (see https://github.com/seanmonstar/warp/issues/388#issuecomment-576453485).
//   A rejection means Warp will fall through to another filter and ultimately hit a rejection
//   handler, with people reporting rejections take way too long to process with more routes.
//
//   In our case, the error in our handler function is final and we also would like to be able
//   to use the ? operator to bail out of the handler if an error exists, which using a Result type
//   handles for us.
//
//   2) ApiError knows how to convert itself to an HTTP response + status code (error-specific), allowing
//   us to implement Reply for ApiError.
//
//   3) We can't implement Reply for Result<Reply, Reply> (we don't control Result), so we have to
//   add a final function `into_response` that converts our Result into a Response. We won't need
//   to do this when https://github.com/seanmonstar/warp/pull/909 is merged:
//
//   ```
//   .then(my_handler_func)
//   .map(into_response)
//   ```
//

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

// Wrap DataFusion errors so that we can automagically return an
// `ApiError(DataFusionError)` by using the `?` operator
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
