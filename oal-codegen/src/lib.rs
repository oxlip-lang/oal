use indexmap::indexmap;

use oal_syntax::ast::TypeRel;
use openapiv3::{Info, OpenAPI};

pub struct Api {
    rels: Vec<TypeRel>,
}

impl Api {
    pub fn new() -> Api {
        Api { rels: Vec::new() }
    }

    pub fn expose_all<'a, I: Iterator<Item = &'a TypeRel>>(&mut self, rels: I) -> &Self {
        self.rels = rels.cloned().collect();
        self
    }

    pub fn render(&self) -> OpenAPI {
        OpenAPI {
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
        }
    }
}
