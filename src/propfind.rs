use exile::Node::Element;
use phf::phf_map;

static PROPS: phf::Map<&'static str, &str> = phf_map! {
    "principal-URL" => "<d:principal-URL><d:href>/principals/mock/</d:href></d:principal-URL>",
    "displayname" => "d:displayname>Linkal</d:displayname>",
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

pub fn generate_response(props: Vec<&str>) -> String {
    props
        .into_iter()
        .map(|prop| match PROPS.get(prop) {
            Some(response) => response,
            None => "",
        })
        .rev()
        .collect::<Vec<&str>>()
        .join("")
}
