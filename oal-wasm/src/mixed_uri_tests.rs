use serde_yaml::Value;

fn document(source: &str) -> Value {
    let result = crate::compile(source);
    assert!(result.error.is_empty(), "{source}: {}", result.error);
    serde_yaml::from_str(&result.api).unwrap()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn mixed_segments_preserve_literals_and_parameter_order() {
    for (uri, expected, names) in [
        (
            "/api/db/{'dataset str}.{'table str}",
            "/api/db/{dataset}.{table}",
            vec!["dataset", "table"],
        ),
        (
            "/files/{'name str}.json",
            "/files/{name}.json",
            vec!["name"],
        ),
        (
            "/v{'version int}/file-{'name str}-{'revision int}.json",
            "/v{version}/file-{name}-{revision}.json",
            vec!["version", "name", "revision"],
        ),
        (
            "/x/{'first str}{'second str}",
            "/x/{first}{second}",
            vec!["first", "second"],
        ),
        ("/x/{'id int}get", "/x/{id}get", vec!["id"]),
        ("/x/{'id int}123", "/x/{id}123", vec!["id"]),
        ("/x/{'id int}%2F%7B%7D", "/x/{id}%2F%7B%7D", vec!["id"]),
        (
            "/x/pre%20{'name str}%7E_end/",
            "/x/pre%20{name}%7E_end/",
            vec!["name"],
        ),
    ] {
        let doc = document(&format!("res {uri} on get -> {{}};"));
        let paths = doc["paths"].as_mapping().unwrap();
        assert_eq!(paths.len(), 1);
        let item = &doc["paths"][expected];
        let params = item["parameters"].as_sequence().unwrap();
        assert_eq!(params.len(), names.len());
        for (param, name) in params.iter().zip(names) {
            assert_eq!(param["name"].as_str(), Some(name));
            assert_eq!(param["in"].as_str(), Some("path"));
            assert_eq!(param["required"].as_bool(), Some(true));
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn mixed_parameters_keep_types_annotations_and_query_parameters() {
    let doc = document("# description: File name\nlet part = 'name str; res /v{'version int}/{part}.json?{'format str} on get -> {};");
    let params = doc["paths"]["/v{version}/{name}.json"]["parameters"]
        .as_sequence()
        .unwrap();
    assert_eq!(params.len(), 3);
    assert_eq!(params[0]["schema"]["type"].as_str(), Some("integer"));
    assert_eq!(params[1]["schema"]["type"].as_str(), Some("string"));
    assert_eq!(params[1]["description"].as_str(), Some("File name"));
    assert_eq!(params[2]["name"].as_str(), Some("format"));
    assert_eq!(params[2]["in"].as_str(), Some("query"));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn append_preserves_segment_boundaries_and_query_replacement() {
    let doc = document(
        "res (concat /v{'version int}/?{'old str} /file-{'name str}.json?{'new str}) on get -> {};",
    );
    let item = &doc["paths"]["/v{version}/file-{name}.json"];
    let params = item["parameters"].as_sequence().unwrap();
    assert_eq!(params.len(), 3);
    assert_eq!(params[2]["name"].as_str(), Some("new"));
    assert_eq!(
        item["get"]["operationId"].as_str(),
        Some("get-vversion-file-name.json")
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn mixed_uri_schema_examples_use_the_same_parts() {
    let doc = document("res /links on get -> {'link /v{'version int `example: 2`}/{'name str `example: report`}.json};");
    let link = &doc["paths"]["/links"]["get"]["responses"]["default"]["content"]
        ["application/json"]["schema"]["properties"]["link"];
    assert_eq!(
        link["example"].as_str(),
        Some("/v_version_integer_/_name_string_.json")
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn old_paths_and_whitespace_keep_their_meaning() {
    for (uri, expected, id) in [
        ("/", "/", "get-root"),
        ("/files/{ 'id int }/", "/files/{id}/", "get-files-id-root"),
        (
            "/files/ /* comment */ { 'id int }/details",
            "/files/{id}/details",
            "get-files-id-details",
        ),
        (
            "/api/static.json",
            "/api/static.json",
            "get-api-static.json",
        ),
        ("/api/./-/_/%2F", "/api/./-/_/%2F", "get-api-.---_-%2f"),
    ] {
        let doc = document(&format!("res {uri} on get -> {{}};"));
        assert!(doc["paths"][expected].is_mapping());
        assert_eq!(
            doc["paths"][expected]["get"]["operationId"].as_str(),
            Some(id)
        );
    }
    let doc = document("let prop obj = 'id str; res /x/{prop {'nested str}}.json on get -> {};");
    assert!(doc["paths"]["/x/{id}.json"].is_mapping());
    assert_eq!(
        document("res /x/{'id str}on get -> {};"),
        document("res /x/{'id str} on get -> {};")
    );
    assert_eq!(
        document("res /x on get -> </a/{'id str}~str>;"),
        document("res /x on get -> </a/{'id str} ~ str>;")
    );
    let doc = document(
        "let prop obj = 'id str; res /v{prop {'link /x/{'nested int}.json}}.txt on get -> {};",
    );
    assert!(doc["paths"]["/v{id}.txt"].is_mapping());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn malformed_mixed_paths_and_nonprimitive_parameters_fail() {
    for input in [
        "res /v{} on get -> {};",
        "res /v{'id str on get -> {};",
        "res /x/{'id str}.{'other str on get -> {};",
        "res /v{'id {}} on get -> {};",
        "res /v{'id [str]} on get -> {};",
        "res /x/{'id str}.{'other {}} on get -> {};",
        "res /x/{'id str}.\\json on get -> {};",
    ] {
        let result = crate::compile(input);
        assert!(!result.error.is_empty(), "accepted {input}");
        assert!(result.api.is_empty());
    }
}
