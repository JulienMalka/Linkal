use crate::{propfind, utils::LinkalError, Calendar, ConversionError, OtherError};
use bytes::Bytes;
use http::{Method, StatusCode};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str};
use ureq;
use warp::http::{Response, Uri};
use warp::Rejection;

pub async fn handle_propfind_locally(
    req_body: Bytes,
    path: &str,
) -> Result<impl warp::Reply, LinkalError> {
    let request_body = str::from_utf8(req_body.as_ref())?;
    let props_requested = propfind::parse_propfind(request_body);
    return Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .status(StatusCode::from_u16(207).unwrap())
        .body(propfind::generate_response(props_requested, &path, true)));
}

pub async fn handle_well_known() -> Result<impl warp::Reply, Infallible> {
    return Ok(warp::redirect(Uri::from_static("/")));
}

pub async fn handle_options() -> Result<impl warp::Reply, Infallible> {
    return Ok(Response::builder()
        .status(StatusCode::from_u16(207).unwrap())
        .header("Content-Type", "text/html; charset=UTF-8")
        .header("DAV", "1, extended-mkcol, access-control")
        .header(
            "ALLOW",
            "OPTIONS, GET, HEAD, DELETE, PROPFIND, PUT, PROPPATCH, COPY, MOVE, REPORT",
        )
        .body(""));
}

pub async fn handle_options_cals() -> Result<impl warp::Reply, Infallible> {
    return Ok(Response::builder()
        .status(StatusCode::from_u16(207).unwrap())
        .header("Content-Type", "text/html; charset=UTF-8")
        .header("DAV", "1, extended-mkcol, access-control, calendar-access")
        .header(
            "ALLOW",
            "OPTIONS, GET, HEAD, DELETE, PROPFIND, PUT, PROPPATCH, COPY, MOVE, REPORT",
        )
        .body(""));
}

pub async fn handle_events(
    path: String,
    req_body: Bytes,
    method: Method,
    calendars: HashMap<String, Calendar>,
    depth: Option<u32>,
) -> Result<impl warp::Reply, LinkalError> {
    let client = ureq::Agent::new();
    let calendar_url = &calendars[&path].url;
    // First we redirect the request to the upstream server
    let response_upstream = client
        .request(method.as_str(), &calendar_url)
        .set("Depth", &depth.unwrap_or(0).to_string())
        .set("Content-Type", "application/xml")
        .send_bytes(propfind::prepare_forwarded_body(&req_body).as_bytes())?
        .into_string()?;

    // Then we have to highjack the response to make it look like we sent it
    // Change the principal owner of the calendar
    let response = propfind::replace_owners(&response_upstream);
    // Change the relative url of the calendar / events
    let response = propfind::replace_relative_urls(&calendars[&path], &response);

    return Ok(Response::builder()
        .status(StatusCode::from_u16(207).unwrap())
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(response));
}

pub async fn handle_proppatch() -> Result<impl warp::Reply, Infallible> {
    return Ok(Response::builder()
            .header("Content-Type", "application/xml; charset=utf-8")
            .status(StatusCode::from_u16(207).unwrap())
            .body(r#"<?xml version="1.0"?>
                     <d:multistatus xmlns:d="DAV:" xmlns:s="http://sabredav.org/ns" xmlns:cal="urn:ietf:params:xml:ns:caldav" xmlns:cs="http://calendarserver.org/ns/" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
                     <d:response>
                        <d:href>/cals/</d:href>
                        <d:propstat>
                            <d:prop>
                            <cal:default-alarm-vevent-date/>
                            </d:prop>
                            <d:status>HTTP/1.1 200 OK</d:status>
                         </d:propstat>
                     </d:response>
                     </d:multistatus>"#.to_owned()));
}

pub async fn handle_cals(
    method: Method,
    calendars: HashMap<String, Calendar>,
    depth: u32,
    req_body: Bytes,
) -> Result<impl warp::Reply, LinkalError> {
    let client = ureq::Agent::new();

    let mut response = r#"<?xml version="1.0"?>
                      <d:multistatus xmlns:d="DAV:" xmlns:s="http://sabredav.org/ns" xmlns:cal="urn:ietf:params:xml:ns:caldav" xmlns:cs="http://calendarserver.org/ns/" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
                      <d:response>
                      <d:href>/cals/</d:href>
                      <d:propstat>
                        <d:prop>
                            <d:resourcetype>
                                <d:collection/>
                            </d:resourcetype>
                        </d:prop>
                        <d:status>HTTP/1.1 200 OK</d:status>
                      </d:propstat>
                      </d:response>"#.to_owned();

    // TODO : make this async
    // For each of the upstream calendars
    for (_, calendar) in calendars {
        // Forward the request to the upstream server
        let response_calendar = client
            .request(method.as_str(), &calendar.url)
            .set("Depth", &depth.to_string())
            .set("Content-Type", "application/xml")
            .send_bytes(req_body.as_ref())?
            .into_string()?;

        // Replace the relative calendar / events url
        let response_calendar = propfind::replace_relative_urls(&calendar, &response_calendar);

        let response_match = Regex::new(r"<d:response>(.*?)</d:response>")
            .unwrap()
            .find(&response_calendar)
            .unwrap();

        // We only want to keep the response part to include it in a bigger response
        let response_calendar = &response_calendar[response_match.start()..response_match.end()];

        response.push_str(&response_calendar);
    }

    response.push_str("</d:multistatus>");

    let response = propfind::replace_owners(&response);

    return Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .status(StatusCode::from_u16(207).unwrap())
        .body(response));
}

pub async fn handle_cals_proppatch(
    req_body: Bytes,
    method: Method,
    calendars: HashMap<String, Calendar>,
) -> Result<impl warp::Reply, Rejection> {
    let client = ureq::Agent::new();

    if method.clone().as_str().eq("PROPPATCH") {
        return Ok(Response::builder()
            .header("Content-Type", "application/xml; charset=utf-8")
            .status(StatusCode::from_u16(207).unwrap())
            .body(r#"
            <?xml version="1.0"?>
            <d:multistatus xmlns:d="DAV:" xmlns:s="http://sabredav.org/ns" xmlns:cal="urn:ietf:params:xml:ns:caldav" xmlns:cs="http://calendarserver.org/ns/" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
                <d:response>
                    <d:href>/cals/</d:href>
                    <d:propstat>
                        <d:prop>
                            <cal:default-alarm-vevent-date/>
                        </d:prop>
                        <d:status>HTTP/1.1 200 OK</d:status>
                    </d:propstat>
                </d:response>
            </d:multistatus>
            "#.to_owned()));
    }

    let mut body = r#"<?xml version="1.0"?>
                <d:multistatus xmlns:d="DAV:" xmlns:s="http://sabredav.org/ns" xmlns:cal="urn:ietf:params:xml:ns:caldav" xmlns:cs="http://calendarserver.org/ns/" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
                <d:response>
                    <d:href>/cals/</d:href>
                    <d:propstat>
                        <d:prop>
                            <d:resourcetype>
                                <d:collection/>
                            </d:resourcetype>
                        </d:prop>
                        <d:status>HTTP/1.1 200 OK</d:status>
                    </d:propstat>
                    <d:propstat>
                        <d:prop>
                            <d:displayname/>
                            <cal:supported-calendar-component-set/>
                        </d:prop>
                        <d:status>HTTP/1.1 404 Not Found</d:status>
                    </d:propstat>
                </d:response>"#.to_owned();

    // TODO : make this async
    for (k, v) in calendars {
        let content = client
            .request(method.as_str(), &v.url)
            .set("Depth", "0")
            .set("Content-Type", "application/xml")
            .send_bytes(req_body.as_ref());

        let finalreq = match content {
            Ok(s) => match s.into_string() {
                Ok(s) => s,
                Err(_) => return Err(warp::reject::custom(ConversionError)),
            },
            Err(_) => return Err(warp::reject::custom(OtherError)),
        };

        // TODO : find a better method here
        let mut to_add = "/remote.php/dav/public-calendars/".to_owned();
        to_add.push_str(&k);

        let mut to_replace = "/cals/".to_owned();
        to_replace.push_str(&k);

        let result = str::replace(&finalreq, &to_add, &to_replace.clone());

        let to_split = result;
        let splitted = to_split.split("<d:response>");
        let vec: Vec<&str> = splitted.collect();

        let intermediate = vec[1];
        let finished = intermediate.split("</d:response>");
        let vec2: Vec<&str> = finished.collect();
        let end = vec2[0];

        let mut response = "<d:response>".to_owned();
        response.push_str(end);
        response.push_str("</d:response>");

        body.push_str(&response);
    }

    body.push_str("</d:multistatus>");

    let re = Regex::new(r"<d:owner>(.*?)</d:owner>").unwrap();
    let before_regex = body.as_str();

    let response = re.replace_all(before_regex, "<d:owner>/principals/mock</d:owner>");
    let response2 = response;
    let re = Regex::new(r"<cs:publish-url>(.*?)</cs:publish-url>").unwrap();
    let response4 = re.replace_all(
        &response2,
        "<cs:publish-url><d:href>http://127.0.0.1/cals/LLWm8qK9iC5YGrrR</d:href></cs:publish-url>",
    );
    let test = response4.into_owned();

    return Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .status(StatusCode::from_u16(207).unwrap())
        .body(test));
}
