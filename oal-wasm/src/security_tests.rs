use serde_yaml::Value;

fn document(input: &str) -> Value {
    let result = crate::compile(input);
    assert!(result.error.is_empty(), "{}", result.error);
    serde_yaml::from_str(&result.api).unwrap()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn operation_security_preserves_requirements_and_scopes() {
    for security in [
        "[]",
        "[{}]",
        "[{bearer: []}]",
        "[{key: []}, {bearer: []}]",
        "[{key: [], oauth: ['read', 'write']}]",
        "[{}, {oauth: ['read']}]",
        r#"[{"\u041a\u043b\u044e\u0447": ["\u0447\u0442\u0435\u043d\u0438\u0435", "quoted \"scope\"", "line\nbreak"]}]"#,
    ] {
        let code = format!("# security: {security}\nlet op = get -> {{}};\nres /secure on op;");
        let doc = document(&code);
        assert_eq!(
            doc["paths"]["/secure"]["get"]["security"],
            serde_yaml::from_str::<Value>(security).unwrap()
        );
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn absent_security_does_not_leak_between_operations() {
    let doc = document(
        r#"
        # security: [{bearer: []}]
        let protected = get -> {};
        res /shared on protected, post -> {};
        res /other on protected;
        res /public on get -> {};
    "#,
    );
    for path in ["/shared", "/other"] {
        assert_eq!(
            doc["paths"][path]["get"]["security"][0]["bearer"],
            Value::Sequence(vec![])
        );
    }
    assert!(!doc["paths"]["/shared"]["post"]
        .as_mapping()
        .unwrap()
        .contains_key("security"));
    assert!(!doc["paths"]["/public"]["get"]
        .as_mapping()
        .unwrap()
        .contains_key("security"));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn later_security_annotation_replaces_instead_of_adding_alternatives() {
    for security in ["[]", "[{second: ['read']}]"] {
        let code = format!("# security: [{{first: []}}]\n# security: {security}\nlet op = get -> {{}};\nres /secure on op;");
        let doc = document(&code);
        assert_eq!(
            doc["paths"]["/secure"]["get"]["security"],
            serde_yaml::from_str::<Value>(security).unwrap()
        );
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn malformed_security_returns_diagnostic_without_output() {
    for security in [
        "null",
        "true",
        "{}",
        "bearer",
        "[null]",
        "[1]",
        "[[]]",
        "[{1: []}]",
        "[{bearer: null}]",
        "[{bearer: read}]",
        "[{bearer: [1]}]",
        "[{bearer: [null]}]",
        "[{bearer: [{}]}]",
        "[{bearer: [true]}]",
        "[{bearer: [!tag read]}]",
        "!tag []",
        "[!tag {}]",
        "[{!tag bearer: []}]",
        "[{bearer: !tag []}]",
    ] {
        let result = crate::compile(&format!(
            "# security: {security}\nlet op = get -> {{}};\nres /secure on op;"
        ));
        assert!(result.api.is_empty(), "accepted {security}");
        assert!(result.error.contains("invalid literal"), "{}", result.error);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn operation_security_overrides_base_without_removing_scheme_definitions() {
    let input = "res /inherit on get -> {};\n# security: []\nlet public = get -> {};\nres /public on public;\n# security: [{bearer: []}]\nlet protected = get -> {};\nres /secure on protected;";
    let main = oal_model::locator::Locator::try_from(crate::INPUT).unwrap();
    let mods = oal_compiler::module::load(&mut crate::WebLoader(input), &main).unwrap();
    let spec = oal_compiler::eval::eval(&mods).unwrap();
    let base = serde_yaml::from_str(
        r#"
openapi: 3.0.3
info: {title: Security, version: '1'}
paths: {}
security: [{key: []}]
components:
  securitySchemes:
    key: {type: apiKey, in: header, name: X-Key}
    bearer: {type: http, scheme: bearer}
"#,
    )
    .unwrap();
    let api = oal_openapi::Builder::new(spec)
        .with_base(base)
        .into_openapi();
    let doc = serde_yaml::to_value(api).unwrap();
    assert_eq!(doc["security"][0]["key"], Value::Sequence(vec![]));
    assert!(!doc["paths"]["/inherit"]["get"]
        .as_mapping()
        .unwrap()
        .contains_key("security"));
    assert_eq!(
        doc["paths"]["/public"]["get"]["security"],
        Value::Sequence(vec![])
    );
    assert_eq!(
        doc["paths"]["/secure"]["get"]["security"][0]["bearer"],
        Value::Sequence(vec![])
    );
    assert_eq!(
        doc["components"]["securitySchemes"]
            .as_mapping()
            .unwrap()
            .len(),
        2
    );
}
