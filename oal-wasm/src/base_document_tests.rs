use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "openapi":"3.0.3",
        "info":{"title":"Example API","version":"1.2.3","description":"Unicode: \u{041f}\u{0440}\u{0438}\u{043c}\u{0435}\u{0440}\nSecond line","contact":{"name":"API team"},"license":{"name":"Apache-2.0"},"x-info":true},
        "servers":[{"url":"https://example.test/api","variables":{"region":{"default":"eu","description":"Region"}}}],
        "paths":{"/old":{"get":{"responses":{"200":{"description":"old"}}}}},
        "components":{
            "schemas":{"Old":{"type":"string"},"Item":{"type":"string"}},
            "securitySchemes":{"key":{"type":"apiKey","in":"header","name":"X-Key"},"bearer":{"type":"http","scheme":"bearer"}},
            "responses":{"Error":{"description":"error"}},
            "parameters":{"Trace":{"name":"X-Trace","in":"header","schema":{"type":"string"}}},
            "examples":{"sample":{"value":{"id":1}}},
            "requestBodies":{"Payload":{"content":{"application/json":{"schema":{"type":"object"}}}}},
            "headers":{"Count":{"schema":{"type":"integer"}}},
            "links":{"Next":{"operationId":"get-items"}},
            "callbacks":{"Callback":{"$ref":"https://example.test/callback.yaml"}},
            "x-components":{"active":true}
        },
        "security":[{"key":[]}],
        "tags":[{"name":"items","description":"Items"}],
        "externalDocs":{"url":"https://example.test/docs"},
        "x-document":{"nested":[true,null,"text"]}
    })
}

fn with_base(input: &str, base: &Value) -> Value {
    let result = crate::compile_with_document(input, serde_json::from_value(base.clone()).unwrap());
    assert!(result.error.is_empty(), "{}", result.error);
    serde_yaml::from_str(&result.api).unwrap()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn preserves_metadata_and_all_non_schema_components() {
    let original = base();
    // Compare typed semantics: the existing CLI serializer emits some defaults.
    let expected =
        serde_json::to_value(serde_json::from_value::<crate::OpenAPI>(original.clone()).unwrap())
            .unwrap();
    let doc = with_base(
        "let @Item = {'id int}; res /items on get -> @Item;",
        &original,
    );
    for (key, value) in expected.as_object().unwrap() {
        if key != "paths" && key != "components" {
            assert_eq!(&doc[key], value, "field {key}");
        }
    }
    for (key, value) in expected["components"].as_object().unwrap() {
        if key != "schemas" {
            assert_eq!(&doc["components"][key], value, "component {key}");
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn generated_paths_and_schemas_replace_base_collections() {
    let original = base();
    let doc = with_base(
        "let @Item = {'id int}; res /items on get -> @Item;",
        &original,
    );
    assert_eq!(doc["paths"].as_object().unwrap().len(), 1);
    assert!(doc["paths"].get("/items").is_some());
    assert!(doc["paths"].get("/old").is_none());
    assert_eq!(doc["components"]["schemas"].as_object().unwrap().len(), 1);
    assert_eq!(doc["components"]["schemas"]["Item"]["type"], "object");
    assert!(doc["components"]["schemas"].get("Old").is_none());
    let empty = with_base("res /empty on get -> {};", &original);
    assert!(empty["components"].get("schemas").is_none());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn supplied_document_has_no_implicit_default_metadata() {
    let minimal = json!({"openapi":"3.0.3","info":{"title":"Minimal","version":"2"},"paths":{}});
    let doc = with_base("res / on get -> {};", &minimal);
    assert_eq!(doc["info"], minimal["info"]);
    assert!(doc.get("servers").is_none());
    assert!(doc.get("security").is_none());
    let defaults = crate::compile("res / on get -> {};");
    let defaults: Value = serde_yaml::from_str(&defaults.api).unwrap();
    assert_eq!(defaults["servers"][0]["url"], "/");
    assert_eq!(defaults["info"]["title"], "OpenAPI definition");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn invalid_base_document_returns_diagnostic_without_output() {
    let original = base();
    let mut missing_info = original.clone();
    missing_info.as_object_mut().unwrap().remove("info");
    let mut bad_schemes = original.clone();
    bad_schemes["components"]["securitySchemes"] = json!({"key":{"type":"not-a-scheme"}});
    for doc in [
        Value::Null,
        json!([]),
        json!("text"),
        json!({}),
        missing_info,
        bad_schemes,
    ] {
        let request =
            crate::parse_document_request(r#"{"source":"res / on get -> {};"}"#, &doc.to_string());
        let result = crate::compilation_result(request.map(|_| String::new()));
        assert!(result.api.is_empty(), "accepted {doc}");
        assert!(
            result.error.starts_with("invalid base document:"),
            "{}",
            result.error
        );
    }
    let result = crate::compile_with_document(
        "res a on get -> {};",
        serde_json::from_value(original).unwrap(),
    );
    assert!(result.api.is_empty());
    assert!(result.error.contains("not in scope"));
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
fn javascript_object_api_matches_typed_rust_and_does_not_mutate_inputs() {
    use wasm_bindgen::JsCast;
    let input = "res /items on get -> {};";
    let doc = js_sys::JSON::parse(&base().to_string()).unwrap();
    let object: js_sys::Object = doc.unchecked_into();
    let before = js_sys::JSON::stringify(&object)
        .unwrap()
        .as_string()
        .unwrap();
    let request: js_sys::Object = js_sys::JSON::parse(&json!({"source":input}).to_string())
        .unwrap()
        .unchecked_into();
    let request_before = js_sys::JSON::stringify(&request)
        .unwrap()
        .as_string()
        .unwrap();
    let result = crate::compile_document(request.clone(), object.clone());
    assert!(result.error.is_empty(), "{}", result.error);
    assert_eq!(
        result.api,
        crate::compile_with_document(input, serde_json::from_str(&before).unwrap()).api
    );
    assert_eq!(
        js_sys::JSON::stringify(&object)
            .unwrap()
            .as_string()
            .unwrap(),
        before
    );
    assert_eq!(
        js_sys::JSON::stringify(&request)
            .unwrap()
            .as_string()
            .unwrap(),
        request_before
    );
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
fn cyclic_javascript_document_returns_diagnostic() {
    use wasm_bindgen::JsCast;
    let doc = js_sys::Object::new();
    js_sys::Reflect::set(&doc, &"self".into(), &doc).unwrap();
    let request = js_sys::JSON::parse(r#"{"source":"res / on get -> {};"}"#)
        .unwrap()
        .unchecked_into();
    let result = crate::compile_document(request, doc);
    assert!(result.api.is_empty());
    assert!(result.error.starts_with("invalid base document:"));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn compilation_envelope_requires_only_a_source_string() {
    let doc = base().to_string();
    for input in [
        "null",
        "[]",
        "{}",
        r#"{"source":null}"#,
        r#"{"source":42}"#,
        r#"{"source":"res / on get -> {};", "unknown":true}"#,
    ] {
        let error = crate::parse_document_request(input, &doc).unwrap_err();
        assert!(error.to_string().starts_with("invalid compilation input:"));
    }
    let (source, base) =
        crate::parse_document_request(r#"{"source":"res / on get -> {};"}"#, &doc).unwrap();
    let result = crate::compile_with_document(&source, base);
    assert!(result.error.is_empty(), "{}", result.error);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn base_document_does_not_leak_into_later_compilations() {
    let input = "res /items on get -> {};";
    let before: Value = serde_yaml::from_str(&crate::compile(input).api).unwrap();
    let first = with_base(input, &base());
    assert_eq!(first["info"]["title"], "Example API");
    let mut next = base();
    next["info"]["title"] = json!("Another API");
    next["security"] = json!([]);
    let second = with_base(input, &next);
    assert_eq!(second["info"]["title"], "Another API");
    assert_eq!(second["security"], json!([]));
    let after: Value = serde_yaml::from_str(&crate::compile(input).api).unwrap();
    assert_eq!(after, before);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test::wasm_bindgen_test]
fn javascript_arguments_are_required_objects_and_errors_do_not_escape() {
    use wasm_bindgen::{JsCast, JsValue};
    let input: js_sys::Object = js_sys::JSON::parse(r#"{"source":"res / on get -> {};"}"#)
        .unwrap()
        .unchecked_into();
    let doc: js_sys::Object = js_sys::JSON::parse(&base().to_string())
        .unwrap()
        .unchecked_into();
    for bad in [
        JsValue::NULL,
        JsValue::UNDEFINED,
        JsValue::from_str("text"),
        JsValue::from_f64(1.0),
        js_sys::Array::new().into(),
    ] {
        let result = crate::compile_document(bad.clone().unchecked_into(), doc.clone());
        assert!(result.api.is_empty());
        assert!(result.error.starts_with("invalid compilation input:"));
        let result = crate::compile_document(input.clone(), bad.unchecked_into());
        assert!(result.api.is_empty());
        assert!(result.error.starts_with("invalid base document:"));
    }
    assert!(crate::compile_document(input, doc).error.is_empty());
}
