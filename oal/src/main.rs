use indexmap::indexmap;
use openapiv3::{Info, OpenAPI};

use oal_compiler::paths;
use oal_syntax::parse;

fn main() {
    let doc = parse("doc.txt");

    println!("{:#?}", doc);

    let _paths = paths(&doc).expect("compilation failed");

    let spec = OpenAPI {
        openapi: "3.0.1".into(),
        info: Info {
            title: "Test OpenAPI specification".into(),
            description: None,
            terms_of_service: None,
            contact: None,
            license: None,
            version: "0.1.0".into(),
            extensions: indexmap! {},
        },
        servers: vec![],
        paths: Default::default(),
        components: None,
        security: None,
        tags: vec![],
        external_docs: None,
        extensions: indexmap! {},
    };

    let output = serde_yaml::to_string(&spec).unwrap();

    println!("{}", output);
}
