use super::compile;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn descriptions_preserve_unicode_separators() {
    for (escaped, expected) in [
        (r"trailing\u0085", "trailing\u{85}"),
        (r"trailing\u2028", "trailing\u{2028}"),
        (r"trailing\u2029", "trailing\u{2029}"),
        (r"line\nnext\u2028", "line\nnext\u{2028}"),
    ] {
        let source = format!("# description: \"{escaped}\"\nlet op = get -> {{}}; res / on op;");
        let result = compile(&source);
        assert!(result.error.is_empty(), "{}", result.error);
        assert!(!result.api.contains(['\u{85}', '\u{2028}', '\u{2029}']));
        let doc: serde_yaml::Value = serde_yaml::from_str(&result.api).unwrap();
        assert_eq!(doc["paths"]["/"]["get"]["description"], expected);
    }
}
