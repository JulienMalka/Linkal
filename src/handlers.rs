use crate::Calendar;
use crate::ConversionError;
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
        <d:href>/principals/ens/</d:href>
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
        <d:href>/principals/ens/calendar-proxy-read/</d:href>
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
    return Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(finalreq));
}

pub async fn handle_cals(
    calendars: HashMap<String, Calendar>,
) -> Result<impl warp::Reply, Infallible> {
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

    for (k, v) in calendars {
        let pattern_instanciated = format!(
            r#"<d:response>
                    <d:href>/cals/{}/</d:href>
                    <d:propstat>
                        <d:prop>
                            <d:displayname>{}</d:displayname>
                            <d:resourcetype>
                                <d:collection/>
                                <cal:calendar/>
                            </d:resourcetype>
                            <cal:supported-calendar-component-set>
                                <cal:comp name="VEVENT"/>
                            </cal:supported-calendar-component-set>
                        </d:prop>
                        <d:status>HTTP/1.1 200 OK</d:status>
                    </d:propstat>
                </d:response>"#,
            k, v.name
        );
        body.push_str(&pattern_instanciated);
    }

    body.push_str("</d:multistatus>");

    return Ok(Response::builder()
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(body));
}
