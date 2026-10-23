use serde_yaml::Value;

fn compile_document(input: &str) -> Value {
    let result = crate::compile(input);
    assert!(result.error.is_empty(), "{}", result.error);
    serde_yaml::from_str(&result.api).unwrap()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn inline_examples_preserve_json_types_in_requests_and_responses() {
    for value in [
        r#"{"id":1,"nested":{"items":[true,null,"text"]}}"#,
        r#"[{"id":1},2,"three"]"#,
        r#"{}"#,
        r#"[]"#,
        "\"\"",
        r#""https://example.test/inline-not-a-url-reference""#,
        r#""quotes: \" and ' and \u0060; slash: \\; newline: \n; unicode: \u041f\u0440\u0438\u043c\u0435\u0440""#,
        "0",
        "-12",
        "3.125",
        "18446744073709551615",
        "true",
        "false",
        "null",
    ] {
        let code = format!(
            r#"res /sample on post : <{{}}> `examples: {{sample: {{value: {value}}}}}` -> <status=200, {{}}> `examples: {{sample: {{value: {value}}}}}`;"#
        );
        let doc = compile_document(&code);
        let operation = &doc["paths"]["/sample"]["post"];
        let expected: Value = serde_yaml::from_str(value).unwrap();
        for example in [
            &operation["requestBody"]["content"]["application/json"]["examples"]["sample"],
            &operation["responses"]["200"]["content"]["application/json"]["examples"]["sample"],
        ] {
            let example = example.as_mapping().expect("expected example object");
            assert_eq!(example.get("value"), Some(&expected), "input: {value}");
            assert!(!example.contains_key("externalValue"));
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn external_examples_remain_external_alongside_inline_values() {
    let doc = compile_document(
        r#"
        res /sample on get -> <status=200, {}> `examples: {remote: "examples/sample.json", inline: {value: "examples/sample.json"}, "\u041f\u0440\u0438\u043c\u0435\u0440 1": {value: null}}`;
    "#,
    );
    let examples = &doc["paths"]["/sample"]["get"]["responses"]["200"]["content"]
        ["application/json"]["examples"];
    assert_eq!(examples["remote"]["externalValue"], "examples/sample.json");
    assert!(!examples["remote"]
        .as_mapping()
        .unwrap()
        .contains_key("value"));
    assert_eq!(examples["inline"]["value"], "examples/sample.json");
    let name = "\u{041f}\u{0440}\u{0438}\u{043c}\u{0435}\u{0440} 1";
    assert_eq!(
        examples[name].as_mapping().unwrap().get("value"),
        Some(&Value::Null)
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn content_examples_override_schema_examples_and_absence_falls_back() {
    let doc = compile_document(
        r#"
        let payload = {} `examples: {inherited: {value: {id: 1}}}`;
        res /sample on post : payload -> <status=200, payload> `examples: {own: {value: [2]}}`;
        res /fallback on get -> <status=200, payload>;
        res /empty on get -> <status=200, payload> `examples: {}`;
    "#,
    );
    let post = &doc["paths"]["/sample"]["post"];
    assert_eq!(
        post["requestBody"]["content"]["application/json"]["examples"]["inherited"]["value"]["id"],
        1
    );
    let own = &post["responses"]["200"]["content"]["application/json"]["examples"];
    assert_eq!(own["own"]["value"][0], 2);
    assert!(!own.as_mapping().unwrap().contains_key("inherited"));
    assert_eq!(
        doc["paths"]["/fallback"]["get"]["responses"]["200"]["content"]["application/json"]
            ["examples"]["inherited"]["value"]["id"],
        1
    );
    assert!(
        !doc["paths"]["/empty"]["get"]["responses"]["200"]["content"]["application/json"]
            .as_mapping()
            .unwrap()
            .contains_key("examples")
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn malformed_examples_return_diagnostics_without_output() {
    for annotation in [
        "examples: []",
        "examples: {sample: 1}",
        "examples: {sample: {}}",
        "examples: {sample: {value: 1, externalValue: 'sample.json'}}",
        "examples: {sample: {value: .inf}}",
        "examples: {sample: {value: {1: 'not a JSON key'}}}",
    ] {
        for body in [
            format!("<{{}}> `{annotation}`"),
            format!("{{}} `{annotation}`"),
        ] {
            let result = crate::compile(&format!("res /sample on get -> {body};"));
            assert!(result.api.is_empty(), "accepted {annotation}");
            assert!(result.error.contains("invalid literal"), "{}", result.error);
        }
    }
}
