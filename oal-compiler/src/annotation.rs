use crate::errors::{Error, Kind, Result};
use crate::spec::{Example, Examples};
use serde_yaml::{Mapping, Sequence, Value};

/// An indexed annotation set.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Annotation {
    pub props: Mapping,
}

/// Extends a value when possible or defaults to overwrite.
fn deep_extend_value(prev: &mut Value, other: Value) {
    if let Value::Mapping(pm) = prev {
        if let Value::Mapping(om) = other {
            return deep_extend_mapping(pm, om);
        }
    } else if let Value::Sequence(ps) = prev {
        if let Value::Sequence(os) = other {
            return deep_extend_sequence(ps, os);
        }
    }
    *prev = other;
}

/// Extends a mapping recursively.
fn deep_extend_mapping(prev: &mut Mapping, other: Mapping) {
    for (k, ov) in other.into_iter() {
        if let Some(pv) = prev.get_mut(&k) {
            deep_extend_value(pv, ov)
        } else {
            prev.insert(k, ov);
        }
    }
}

/// Extends a sequence by concatenation.
fn deep_extend_sequence(prev: &mut Sequence, other: Sequence) {
    prev.extend(other);
}

#[test]
fn test_deep_extend() {
    let mut m1 = serde_yaml::from_str(r#"{ a: { x: 0 }, b: 1, c: [1] }"#).unwrap();
    let m2 = serde_yaml::from_str(r#"{ a: { y: 0 }, b: 2, c: [2] }"#).unwrap();
    let exp = serde_yaml::from_str::<Mapping>(r#"{ a: { x: 0, y: 0 }, b: 2, c: [1,2] }"#).unwrap();

    deep_extend_mapping(&mut m1, m2);

    assert_eq!(m1, exp);
}

impl Annotation {
    /// Extends the set by consuming annotations from the other set.
    pub fn extend(&mut self, other: Self) {
        deep_extend_mapping(&mut self.props, other.props);
    }

    pub fn get_str(&self, s: &str) -> Option<&str> {
        self.props
            .get(Value::String(s.to_owned()))
            .and_then(Value::as_str)
    }

    pub fn get_string(&self, s: &str) -> Option<String> {
        self.get_str(s).map(ToOwned::to_owned)
    }

    pub fn get_bool(&self, s: &str) -> Option<bool> {
        self.props
            .get(Value::String(s.to_owned()))
            .and_then(Value::as_bool)
    }

    pub fn get_num(&self, s: &str) -> Option<f64> {
        self.props
            .get(Value::String(s.to_owned()))
            .and_then(Value::as_f64)
    }

    pub fn get_int(&self, s: &str) -> Option<i64> {
        self.props
            .get(Value::String(s.to_owned()))
            .and_then(Value::as_i64)
    }

    pub fn get_size(&self, s: &str) -> Option<usize> {
        self.props
            .get(Value::String(s.to_owned()))
            .and_then(Value::as_u64)
            .map(|u| u as usize)
    }

    pub fn get_enum(&self, s: &str) -> Option<Vec<String>> {
        self.props
            .get(Value::String(s.to_owned()))
            .and_then(Value::as_sequence)
            .map(|seq| {
                seq.iter()
                    .flat_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
    }

    pub fn get_examples(&self) -> Result<Option<Examples>> {
        let Some(value) = self.props.get("examples") else {
            return Ok(None);
        };
        let mapping = value
            .as_mapping()
            .ok_or_else(|| Error::new(Kind::InvalidLiteral, "examples must be a mapping"))?;
        let mut examples = Examples::new();
        for (key, value) in mapping {
            let name = key
                .as_str()
                .ok_or_else(|| Error::new(Kind::InvalidLiteral, "example names must be strings"))?;
            let example = match value {
                Value::String(url) => Example::External(url.clone()),
                Value::Mapping(fields) if fields.len() == 1 && fields.contains_key("value") => {
                    let value = json_example_value(&fields["value"]).map_err(|error| {
                        Error::new(Kind::InvalidLiteral, format!("example {name:?}: {error}"))
                    })?;
                    Example::Inline(value)
                }
                _ => {
                    return Err(Error::new(
                        Kind::InvalidLiteral,
                        format!(
                        "example {name:?} must be a URL string or a mapping containing only value"
                    ),
                    ))
                }
            };
            examples.insert(name.to_owned(), example);
        }
        Ok(Some(examples))
    }
}

/// Reject YAML-only values instead of coercing keys or turning NaN into JSON null.
fn json_example_value(value: &Value) -> std::result::Result<serde_json::Value, &'static str> {
    use serde_json::Value as Json;
    Ok(match value {
        Value::Null => Json::Null,
        Value::Bool(value) => Json::Bool(*value),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                value.into()
            } else if let Some(value) = value.as_u64() {
                value.into()
            } else {
                serde_json::Number::from_f64(value.as_f64().ok_or("invalid JSON number")?)
                    .ok_or("example numbers must be finite")?
                    .into()
            }
        }
        Value::String(value) => Json::String(value.clone()),
        Value::Sequence(values) => Json::Array(
            values
                .iter()
                .map(json_example_value)
                .collect::<std::result::Result<_, _>>()?,
        ),
        Value::Mapping(values) => {
            let mut object = serde_json::Map::new();
            for (key, value) in values {
                let key = key.as_str().ok_or("example object keys must be strings")?;
                object.insert(key.to_owned(), json_example_value(value)?);
            }
            Json::Object(object)
        }
        Value::Tagged(_) => return Err("YAML tags are not JSON example values"),
    })
}

impl TryFrom<&str> for Annotation {
    type Error = serde_yaml::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let props = serde_yaml::from_str(format!("{{ {value} }}").as_str())?;
        Ok(Annotation { props })
    }
}

#[test]
fn test_examples() -> anyhow::Result<()> {
    let ann = Annotation::try_from(
        r#"examples: { remote: "examples/sample.json", object: {value: {id: 1}}, nothing: {value: null} }"#,
    )?;
    let examples = ann.get_examples()?.unwrap();
    assert_eq!(
        examples["remote"],
        Example::External("examples/sample.json".into())
    );
    assert_eq!(
        examples["object"],
        Example::Inline(serde_json::json!({"id": 1}))
    );
    assert_eq!(
        examples["nothing"],
        Example::Inline(serde_json::Value::Null)
    );
    assert!(Annotation::default().get_examples()?.is_none());
    assert_eq!(
        Annotation::try_from("examples: {}")?.get_examples()?,
        Some(Examples::new())
    );
    Ok(())
}

#[test]
fn test_invalid_examples() -> anyhow::Result<()> {
    for input in [
        "examples: []",
        "examples: {1: 'sample.json'}",
        "examples: {sample: {}}",
        "examples: {sample: {value: 1, externalValue: 'sample.json'}}",
        "examples: {sample: {value: .nan}}",
        "examples: {sample: {value: [.inf]}}",
        "examples: {sample: {value: {1: 'not a JSON key'}}}",
        "examples: {sample: {value: !tag 'not JSON'}}",
    ] {
        let ann = Annotation::try_from(input)?;
        assert!(ann.get_examples().is_err(), "accepted {input}");
    }
    Ok(())
}
