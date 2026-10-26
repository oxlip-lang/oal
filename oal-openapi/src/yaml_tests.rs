use super::to_yaml;
use serde_json::{json, Value};

#[test]
fn string_keys_and_values_survive_yaml_readers() {
    for text in [
        "plain",
        "",
        "true",
        "null",
        "1e1000",
        "yes",
        "---",
        "...",
        "<<",
        ": value",
        " leading",
        "trailing ",
        "\tcontrol\t",
        "CR\rLF\n",
        "trailing\n",
        "\u{85}",
        "\u{2028}",
        "\u{2029}",
        "trailing\u{85}",
        "trailing\u{2028}",
        "trailing\u{2029}",
        "a\u{2028}b\u{2029}c",
        "\u{feff}BOM",
        "\0",
        "quotes: \" ' ` \\",
        "\u{418}\u{43c}\u{44f} \u{1f680}",
    ] {
        let doc = json!({"mapping": {(text): text}, "list": [text], "description": text});
        let yaml = to_yaml(&doc).unwrap();
        // YAML 1.1 and 1.2 disagree on raw NEL/LS/PS. Escapes remove ambiguity.
        assert!(
            !yaml.contains(['\u{85}', '\u{2028}', '\u{2029}']),
            "{text:?}: {yaml}"
        );
        assert_eq!(
            serde_yaml::from_str::<Value>(&yaml).unwrap(),
            doc,
            "{text:?}"
        );
    }
}

#[test]
fn yaml_preserves_scalar_types_and_empty_collections() {
    let doc = json!({"object": {}, "array": [], "null": null, "boolean": false,
        "integer": 42, "float": 1.25, "strings": ["42", "false", "null"],
        "nested": [{"value": [true, null, {}]}]});
    let yaml = to_yaml(&doc).unwrap();
    assert_eq!(serde_yaml::from_str::<Value>(&yaml).unwrap(), doc);
}
