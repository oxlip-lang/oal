use crate::errors::Result;
use crate::node::NodeMut;
use crate::scope::Env;
use oal_syntax::ast::AsExpr;
use serde_yaml::{Mapping, Value};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Annotation {
    pub props: Mapping,
}

impl Annotation {
    pub fn extend(&mut self, other: Self) {
        self.props.extend(other.props.into_iter());
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

pub trait Annotated {
    fn annotation(&self) -> Option<&Annotation>;
    fn annotate(&mut self, a: Annotation);
}

fn compose(acc: &mut Option<Annotation>, text: &str) -> Result<()> {
    let addon = Annotation::try_from(text)?;
    acc.get_or_insert(Default::default()).extend(addon);
    Ok(())
}

fn assign<T: Annotated>(acc: &mut Option<Annotation>, e: &mut T) {
    if let Some(ann) = acc.take() {
        e.annotate(ann);
    }
}

pub fn annotate<T>(acc: &mut Option<Annotation>, _env: &mut Env<T>, node: NodeMut<T>) -> Result<()>
where
    T: AsExpr + Annotated,
{
    match node {
        NodeMut::Expr(e) => {
            if let Some(a) = &e.as_node().ann {
                compose(acc, &a.text)?;
            }
            assign(acc, e);
        }
        NodeMut::Ann(a) => {
            compose(acc, &a.text)?;
        }
        NodeMut::Decl(d) => {
            assign(acc, &mut d.expr);
        }
        _ => {}
    }
    Ok(())
}
