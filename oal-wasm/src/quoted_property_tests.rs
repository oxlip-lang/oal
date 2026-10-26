use super::compile;
use serde_yaml::Value;

fn document(source: &str) -> Value {
    let result = compile(source);
    assert!(result.error.is_empty(), "{}", result.error);
    serde_yaml::from_str(&result.api).unwrap()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn quoted_properties_preserve_json_names() {
    let doc = document(
        r#"res / on get -> <status=200, {
        'bare! str,
        '"user.name"! str,
        '"display name"? str,
        '"\u0418\u043c\u044f" str,
        '"quote\"slash\\line\n\t\u2028\ud83d\ude80" str,
        '"" str,
        '"__proto__" str
    }>;"#,
    );
    let schema =
        &doc["paths"]["/"]["get"]["responses"]["200"]["content"]["application/json"]["schema"];
    let props = schema["properties"].as_mapping().unwrap();
    for name in [
        "bare",
        "user.name",
        "display name",
        "\u{418}\u{43c}\u{44f}",
        "quote\"slash\\line\n\t\u{2028}\u{1f680}",
        "",
        "__proto__",
    ] {
        assert!(props.contains_key(Value::from(name)), "missing {name:?}");
    }
    assert!(schema["required"]
        .as_sequence()
        .unwrap()
        .contains(&Value::from("user.name")));
    assert!(!schema["required"]
        .as_sequence()
        .unwrap()
        .contains(&Value::from("display name")));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn quoted_names_work_in_parameters_and_headers() {
    let doc = document(
        r#"res /items/{'"item.id" int} on
        get {'"filter.name" str}: <headers={'"X-Request-ID" str}> ->
        <status=200, headers={'"X-Response-ID" str}, {'"result.name" str}>;"#,
    );
    let path = &doc["paths"]["/items/{item.id}"];
    assert_eq!(path["parameters"][0]["name"], "item.id");
    let op = &path["get"];
    let params = op["parameters"].as_sequence().unwrap();
    for (name, location) in [("filter.name", "query"), ("X-Request-ID", "header")] {
        assert!(params
            .iter()
            .any(|p| p["name"] == name && p["in"] == location));
    }
    assert!(op["responses"]["200"]["headers"]["X-Response-ID"].is_mapping());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn quoted_and_bare_names_are_equivalent() {
    assert_eq!(
        document("res / on get -> {'name! str};"),
        document(r#"res / on get -> {'"name"! str};"#)
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn malformed_quoted_names_report_errors() {
    for name in [
        r#"'"\q""#,
        r#"'"\ud800""#,
        r#"'"\udc00""#,
        r#"'"\uZZZZ""#,
        r#"'"unfinished"#,
        "'\"raw\nnewline\"",
    ] {
        let result = compile(&format!("res / on get -> {{{name} str}};"));
        assert!(!result.error.is_empty(), "{name}");
        assert!(result.api.is_empty(), "{name}");
    }
}
