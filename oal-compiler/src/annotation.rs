use crate::errors::Result;
use crate::node::NodeMut;
use crate::scope::Env;
use oal_syntax::ast::AsExpr;
use serde_yaml::{Mapping, Value};

/// An indexed annotation set.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Annotation {
    pub props: Mapping,
}

/// Extends a mapping recursively.
fn deep_extend(prev: &mut Mapping, other: Mapping) {
    for (k, ov) in other.into_iter() {
        if let Some(Value::Mapping(pm)) = prev.get_mut(&k) {
            if let Value::Mapping(om) = ov {
                deep_extend(pm, om);
            } else {
                prev.insert(k, ov);
            }
        } else {
            prev.insert(k, ov);
        }
    }
}

#[test]
fn test_deep_extend() {
    let mut m1 = serde_yaml::from_str(r#"{ a: { x: 0 }, b: 1, c: {} }"#).unwrap();
    let m2 = serde_yaml::from_str(r#"{ a: { y: 0 }, b: 2, c: true }"#).unwrap();
    let exp = serde_yaml::from_str::<Mapping>(r#"{ a: { x: 0, y: 0 }, b: 2, c: true }"#).unwrap();

    deep_extend(&mut m1, m2);

    assert_eq!(m1, exp);
}

impl Annotation {
    /// Extends the set by consuming annotations from the other set.
    pub fn extend(&mut self, other: Self) {
        deep_extend(&mut self.props, other.props);
    }

    pub fn get_str(&self, s: &str) -> Option<&str> {
        self.props
            .get(&Value::String(s.to_owned()))
            .and_then(Value::as_str)
    }

    pub fn get_string(&self, s: &str) -> Option<String> {
        self.get_str(s).map(ToOwned::to_owned)
    }

    pub fn get_bool(&self, s: &str) -> Option<bool> {
        self.props
            .get(&Value::String(s.to_owned()))
            .and_then(Value::as_bool)
    }

    pub fn get_num(&self, s: &str) -> Option<f64> {
        self.props
            .get(&Value::String(s.to_owned()))
            .and_then(Value::as_f64)
    }

    pub fn get_int(&self, s: &str) -> Option<i64> {
        self.props
            .get(&Value::String(s.to_owned()))
            .and_then(Value::as_i64)
    }

    pub fn get_enum(&self, s: &str) -> Option<Vec<String>> {
        self.props
            .get(&Value::String(s.to_owned()))
            .and_then(Value::as_sequence)
            .map(|seq| {
                seq.iter()
                    .flat_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
    }
}

impl TryFrom<&str> for Annotation {
    type Error = serde_yaml::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let props = serde_yaml::from_str(format!("{{ {} }}", value).as_str())?;
        Ok(Annotation { props })
    }
}

/// Expressions that support annotations.
pub trait Annotated {
    fn annotation(&self) -> Option<&Annotation>;
    fn annotate(&mut self, a: Annotation);
}

/// Parses a piece of text and accumulates the resulting annotation.
fn compose(acc: &mut Option<Annotation>, text: &str) -> Result<()> {
    let addon = Annotation::try_from(text)?;
    acc.get_or_insert_with(Default::default).extend(addon);
    Ok(())
}

/// Assigns the accumulated annotation to the given expression.
fn assign<T: Annotated>(acc: &mut Option<Annotation>, e: &mut T) {
    if let Some(ann) = acc.take() {
        e.annotate(ann);
    }
}

/// Visits an abstract syntax tree to process annotations.
pub fn annotate<T>(
    acc: &mut Option<Annotation>,
    _env: &mut Env<T>,
    node_ref: NodeMut<T>,
) -> Result<()>
where
    T: AsExpr + Annotated,
{
    match node_ref {
        NodeMut::Expr(expr) => {
            if let Some(a) = &expr.as_node().ann {
                compose(acc, &a.text).map_err(|err| err.at(a.span))?;
            }
            assign(acc, expr);
        }
        NodeMut::Ann(a) => {
            compose(acc, &a.text).map_err(|err| err.at(a.span))?;
        }
        NodeMut::Decl(d) => {
            assign(acc, &mut d.expr);
        }
        _ => {}
    }
    Ok(())
}
