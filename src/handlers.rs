use crate::propfind;
use crate::utils::LinkalError;
use crate::Calendar;
use crate::ConversionError;
use crate::OtherError;
use bytes::Bytes;
use http::Method;
use http::StatusCode;
use regex::Regex;
use std::collections::HashMap;
use std::convert::Infallible;
use std::str;
use ureq;
use warp::http::Response;
use warp::http::Uri;
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
) -> Result<impl warp::Reply, Rejection> {
    let client = ureq::Agent::new();
    let cal_url = &calendars[&path].url;
    let body = req_body;
    let re = Regex::new(r"cals").unwrap();
    let newbody = re
        .replace_all(
            str::from_utf8(&body).unwrap(),
            "remote.php/dav/public-calendars",
        )
        .into_owned();

    let content = client
        .request(method.as_str(), &cal_url)
        .set("Depth", &depth.unwrap_or(0).to_string())
        .set("Content-Type", "application/xml")
        .send_bytes(newbody.as_bytes());

    let response = match content {
        Ok(s) => match s.into_string() {
            Ok(s) => s,
            Err(_) => return Err(warp::reject::custom(ConversionError)),
        },
        Err(_) => return Err(warp::reject::custom(OtherError)),
    };

    let end_url = &cal_url.split("/").collect::<Vec<&str>>()[3..];
    let end_url = end_url.join("/");
    let re = Regex::new(r"<d:owner>(.*?)</d:owner>").unwrap();
    let response2 = re.replace_all(&response, "<d:owner>/principals/mock</d:owner>");
    let response3 = response2.into_owned();
    let re = Regex::new(r"<cs:publish-url>(.*?)</cs:publish-url>").unwrap();
    let response4 = re.replace_all(
        &response3,
        "<cs:publish-url><d:href>http://127.0.0.1/cals/LLWm8qK9iC5YGrrR</d:href></cs:publish-url>",
    );
    let response5 = response4.into_owned();

    let re = Regex::new(&end_url).unwrap();
    let response6 = re
        .replace_all(&response5, format!("cals/{}", path))
        .into_owned();

    let re = Regex::new(r"<oc:owner-principal>(.*?)</oc:owner-principal>").unwrap();
    let response7 = re.replace_all(
        &response6,
        "<oc:owner-principal>/principals/mock/</oc:owner-principal>",
    );
    let test = response7.into_owned();

    return Ok(Response::builder()
        .status(StatusCode::from_u16(207).unwrap())
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(test));
}

pub async fn handle_cals(
    method: Method,
    calendars: HashMap<String, Calendar>,
    depth: u32,
    req_body: Bytes,
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
            .set("Depth", &depth.to_string())
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

    let re = Regex::new(r"<oc:owner-principal>(.*?)</oc:owner-principal>").unwrap();
    let response5 = re.replace_all(
        &test,
        "<oc:owner-principal>/principals/mock/</oc:owner-principal>",
    );
    let test = response5.into_owned();

    return Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .status(StatusCode::from_u16(207).unwrap())
        .body(test));
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
