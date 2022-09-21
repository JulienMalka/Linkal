use crate::Calendar;
use crate::ConversionError;
use bytes::Bytes;
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::Infallible;
use ureq;
use warp::http::Response;
use warp::Rejection;

pub async fn handle_index() -> Result<impl warp::Reply, Infallible> {
    return Ok(Response::builder()
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:s="http://sabredav.org/ns" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
    <d:response>
        <d:href>/</d:href>
        <d:propstat>
            <d:prop>
                <d:current-user-principal>
                    <d:href>/principals/mock/</d:href>
                </d:current-user-principal>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
    <d:response>
        <d:href>/principals/</d:href>
        <d:propstat>
            <d:prop>
                <d:current-user-principal>
                    <d:href>/principals/mock/</d:href>
                </d:current-user-principal>
            </d:prop>
           </d:propstat>
           </d:response>
        </d:multistatus>"#));
}

pub async fn handle_home_url() -> Result<impl warp::Reply, Infallible> {
    return Ok(Response::builder()
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:s="http://sabredav.org/ns" xmlns:cal="urn:ietf:params:xml:ns:caldav" xmlns:cs="http://calendarserver.org/ns/" xmlns:card="urn:ietf:params:xml:ns:carddav" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
    <d:response>
        <d:href>/principals/mock/</d:href>
        <d:propstat>
            <d:prop>
                <cal:calendar-home-set>
                    <d:href>/cals/</d:href>
                </cal:calendar-home-set>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
    <d:response>
        <d:href>/principals/mock/calendar-proxy-read/</d:href>
        <d:propstat>
            <d:prop>
                <cal:calendar-home-set>
                    <d:href>/cals/</d:href>
                </cal:calendar-home-set>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
</d:multistatus>"#));
}

pub static CALENDAR_EVENTS_REQUEST: &str = r#"
    <c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
        <d:prop>
            <d:getetag />
            <c:calendar-data />
        </d:prop>
        <c:filter>
            <c:comp-filter name="VCALENDAR">
                <c:comp-filter name="VEVENT" />
            </c:comp-filter>
        </c:filter>
    </c:calendar-query>
"#;

pub async fn handle_events(
    path: String,
    calendars: HashMap<String, Calendar>,
) -> Result<impl warp::Reply, Rejection> {
    let client = ureq::Agent::new();
    let content = client
        .request("REPORT", &calendars[&path].url)
        .set("Depth", "1")
        .set("Content-Type", "application/xml")
        .send_bytes(CALENDAR_EVENTS_REQUEST.as_bytes());

    let finalreq = match content {
        Ok(s) => match s.into_string() {
            Ok(s) => s,
            Err(_) => return Err(warp::reject::custom(ConversionError)),
        },
        Err(_) => return Err(warp::reject::custom(ConversionError)),
    };

    let to_split = finalreq;
    let splitted = to_split.split("<d:owner>");
    let vec: Vec<&str> = splitted.collect();
    let mut beginning = vec[0].to_owned();
    //let mut beg_ref = &beginning;
    if vec.len() > 1 {
        let end = vec[1];
        let finished = end.split("</d:owner>");
        let vec2: Vec<&str> = finished.collect();
        let end2 = vec2[1];

        let together = format!(
            "{}{}{}",
            beginning, "<d:owner>/principals/mock/</d:owner>", end2
        );
        return Ok(Response::builder()
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(together));
    }

    return Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(beginning));
}

//pub fn replace_owners(pieces: Vec<&str>) -> String {
//    let result = "".to_owned();
//    let i = 0;
//    for element in pieces {}
//}

pub async fn handle_cals(
    req_body: Bytes,
    calendars: HashMap<String, Calendar>,
) -> Result<impl warp::Reply, Rejection> {
    let client = ureq::Agent::new();

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
            .request("PROPFIND", &v.url)
            .set("Depth", "0")
            .set("Content-Type", "application/xml")
            .send_bytes(req_body.as_ref());

        let finalreq = match content {
            Ok(s) => match s.into_string() {
                Ok(s) => s,
                Err(_) => return Err(warp::reject::custom(ConversionError)),
            },
            Err(_) => return Err(warp::reject::custom(ConversionError)),
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

    return Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(response));
}
