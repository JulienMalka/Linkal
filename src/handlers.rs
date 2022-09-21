use crate::Calendar;
use crate::ConversionError;
use crate::OtherError;
use bytes::Bytes;
use http::Method;
use http::StatusCode;
use regex::Regex;
use std::collections::HashMap;
use std::convert::Infallible;
use ureq;
use warp::http::Response;
use warp::http::Uri;
use warp::Rejection;

pub async fn handle_index() -> Result<impl warp::Reply, Infallible> {
    return Ok(Response::builder()
            .header("Content-Type", "application/xml; charset=utf-8")
            .status(StatusCode::from_u16(207).unwrap())
            .body(r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:s="http://sabredav.org/ns" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
    <d:response>
        <d:href>/remote.php/dav/</d:href>
        <d:propstat>
            <d:prop>
                <d:resourcetype>
                    <d:collection/>
                </d:resourcetype>
                <d:current-user-principal>
                    <d:href>/principals/mock</d:href>
                </d:current-user-principal>
                <d:current-user-privilege-set>
                    <d:privilege>
                        <d:all/>
                    </d:privilege>
                    <d:privilege>
                        <d:read/>
                    </d:privilege>
                    <d:privilege>
                        <d:write/>
                    </d:privilege>
                    <d:privilege>
                        <d:write-properties/>
                    </d:privilege>
                    <d:privilege>
                        <d:write-content/>
                    </d:privilege>
                    <d:privilege>
                        <d:unlock/>
                    </d:privilege>
                    <d:privilege>
                        <d:bind/>
                    </d:privilege>
                    <d:privilege>
                        <d:unbind/>
                    </d:privilege>
                    <d:privilege>
                        <d:read-acl/>
                    </d:privilege>
                    <d:privilege>
                        <d:read-current-user-privilege-set/>
                    </d:privilege>
                </d:current-user-privilege-set>
            </d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
        <d:propstat>
            <d:prop>
                <d:owner/>
                <d:displayname/>
                <x1:calendar-color xmlns:x1="http://apple.com/ns/ical/"/>
                <x2:calendar-home-set xmlns:x2="urn:ietf:params:xml:ns:caldav"/>
            </d:prop>
            <d:status>HTTP/1.1 404 Not Found</d:status>
        </d:propstat>
    </d:response>
</d:multistatus>"#));
}

pub async fn handle_well_known() -> Result<impl warp::Reply, Infallible> {
    return Ok(warp::redirect(Uri::from_static("/")));
}

pub async fn handle_home_url() -> Result<impl warp::Reply, Infallible> {
    return Ok(Response::builder()
            .header("Content-Type", "application/xml; charset=utf-8")
            .status(StatusCode::from_u16(207).unwrap())
            .body(r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:s="http://sabredav.org/ns" xmlns:cal="urn:ietf:params:xml:ns:caldav" xmlns:cs="http://calendarserver.org/ns/" xmlns:card="urn:ietf:params:xml:ns:carddav" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
    <d:response>
        <d:href>/principals/mock/</d:href>
        <d:propstat>
            <d:prop>
        <d:resourcetype>
                    <d:principal/>
                </d:resourcetype>
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

pub async fn handle_events(
    path: String,
    req_body: Bytes,
    method: Method,
    calendars: HashMap<String, Calendar>,
) -> Result<impl warp::Reply, Rejection> {
    let client = ureq::Agent::new();
    let content = client
        .request(method.as_str(), &calendars[&path].url)
        .set("Depth", "1")
        .set("Content-Type", "application/xml")
        .send_bytes(req_body.as_ref());

    let response = match content {
        Ok(s) => match s.into_string() {
            Ok(s) => s,
            Err(_) => return Err(warp::reject::custom(ConversionError)),
        },
        Err(_) => return Err(warp::reject::custom(ConversionError)),
    };

    let re = Regex::new(r"<d:owner>(.*?)</d:owner>").unwrap();
    let response2 = re.replace_all(&response, "<d:owner>/principals/mock</d:owner>");
    let response3 = response2.into_owned();
    let re = Regex::new(r"<cs:publish-url>(.*?)</cs:publish-url>").unwrap();
    let response4 = re.replace_all(
        &response3,
        "<cs:publish-url><d:href>http://127.0.0.1/cals/LLWm8qK9iC5YGrrR</d:href></cs:publish-url>",
    );
    let response5 = response4.into_owned();

    return Ok(Response::builder()
        .status(StatusCode::from_u16(207).unwrap())
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(response5));
}

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
            .set("Depth", "1")
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
    let response2 = response.into_owned();
    let re = Regex::new(r"<cs:publish-url>(.*?)</cs:publish-url>").unwrap();
    let response4 = re.replace_all(
        &response2,
        "<cs:publish-url><d:href>http://127.0.0.1/cals/LLWm8qK9iC5YGrrR</d:href></cs:publish-url>",
    );
    let response5 = response4.into_owned();

    return Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .status(StatusCode::from_u16(207).unwrap())
        .body(response5));
}
