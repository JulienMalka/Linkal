use exile::Node::Element;
use phf::phf_map;

static PROPS: phf::Map<&'static str, &str> = phf_map! {
    "principal-URL" => "<d:principal-URL><d:href>/principals/mock/</d:href></d:principal-URL>",
    "displayname" => "<d:displayname>Linkal</d:displayname>",
    "calendar-home-set" => "<cal:calendar-home-set><d:href>/cals/</d:href></cal:calendar-home-set>",
    "current-user-principal" => "<d:current-user-principal><d:href>/principals/mock/</d:href></d:current-user-principal>",
    "email-address-set" => "<cs:email-address-set><cs:email-address>hello@linkal.fr</cs:email-address></cs:email-address-set>",
   "supported-report-set" => "<d:supported-report-set><d:supported-report><d:report><d:expand-property/></d:report></d:supported-report><d:supported-report><d:report><d:principal-match/></d:report></d:supported-report><d:supported-report><d:report><d:principal-property-search/></d:report></d:supported-report><d:supported-report><d:report><d:principal-search-property-set/></d:report></d:supported-report><d:supported-report><d:report><oc:filter-comments/></d:report></d:supported-report><d:supported-report><d:report><oc:filter-files/></d:report></d:supported-report></d:supported-report-set>",
};

pub fn parse_propfind(request: &str) -> Vec<String> {
    let tree = exile::parse(request).unwrap();
    let root = tree.root();
    let prop: &exile::Node = root.first_node().unwrap();
    let mut result: Vec<String> = Vec::new();

    let fields = match prop {
        Element(e) => e.children(),
        _ => panic!("Propfind request is not correctly formed"),
    };

    for field in fields {
        result.push(field.name().to_string());
    }
    return result;
}

pub fn generate_response(props: Vec<String>, path: &str) -> String {
    let mut template_start: String = r#"<?xml version="1.0"?>
    <d:multistatus xmlns:d="DAV:" xmlns:s="http://sabredav.org/ns" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
    <d:response>
        <d:href>"#.to_owned();

    let template_middle = r#"</d:href>
        <d:propstat>
            <d:prop>"#;

    let template_end: String = r#"</d:prop>
            <d:status>HTTP/1.1 200 OK</d:status>
        </d:propstat>
    </d:response>
    </d:multistatus>
    "#
    .to_string();

    let props_res = props
        .into_iter()
        .map(|prop| match PROPS.get(&prop) {
            Some(response) => response,
            None => "",
        })
        .rev()
        .collect::<Vec<&str>>()
        .join("");

    template_start.push_str(path);
    template_start.push_str(template_middle);
    template_start.push_str(&props_res);
    template_start.push_str(&template_end);
    template_start
}
