use crate::{propfind, utils::LinkalError, Calendar};
use bytes::Bytes;
use http::{Method, StatusCode};
use std::{collections::HashMap, convert::Infallible, str};
use ureq;
use warp::http::{Response, Uri};

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

pub async fn handle_options(calendar_access: bool) -> Result<impl warp::Reply, Infallible> {
    let mut dav_header = "1, extended-mkcol, access-control".to_owned();

    if calendar_access {
        dav_header.push_str(" calendar-access");
    }

    return Ok(Response::builder()
        .status(StatusCode::from_u16(207).unwrap())
        .header("Content-Type", "text/html; charset=UTF-8")
        .header("DAV", dav_header)
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

    let response = propfind::replace_name(&response, &calendars[&path].name);

    let response = match &calendars[&path].color {
        Some(c) => propfind::replace_color(&response, &c),
        None => response,
    };

    return Ok(Response::builder()
        .status(StatusCode::from_u16(207).unwrap())
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(response));
}

pub async fn handle_cals(
    method: Method,
    calendars: HashMap<String, Calendar>,
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
    //For each of the upstream calendars
    for (_, calendar) in calendars {
        // Forward the request to the upstream server
        let response_calendar = client
            .request(method.as_str(), &calendar.url)
            .set("Depth", "0")
            .set("Content-Type", "application/xml")
            .send_bytes(req_body.as_ref())?
            .into_string()?;

        let response_calendar = propfind::replace_relative_urls(&calendar, &response_calendar);
        let response_calendar = propfind::replace_owners(&response_calendar);
        let response_calendar = propfind::replace_name(&response_calendar, &calendar.name);

        let response_calendar = match calendar.color {
            Some(c) => propfind::replace_color(&response_calendar, &c),
            None => response_calendar,
        };

        let response_dom = exile::parse(response_calendar)?
            .root()
            .first_node()
            .unwrap()
            .to_owned();

        let response_dom = match response_dom {
            exile::Node::Element(e) => e,
            _ => panic!("Not a node"),
        };

        response.push_str(
            exile::Document::from_root(response_dom)
                .to_string()
                .as_str(),
        );
    }

    response.push_str("</d:multistatus>");

    let response = propfind::replace_owners(&response);
    return Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .status(StatusCode::from_u16(207).unwrap())
        .body(response));
}
