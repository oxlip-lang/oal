use crate::errors::Result;
use crate::scope::Env;
use oal_syntax::ast::{AsExpr, NodeMut};

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    pub props: serde_yaml::Mapping,
}

impl Annotation {
    pub fn get_string(&self, s: &str) -> Option<String> {
        self.props
            .get(&serde_yaml::Value::String(s.to_owned()))
            .and_then(|a| a.as_str())
            .map(|a| a.to_owned())
    }
}

pub trait Annotated {
    fn annotation(&self) -> Option<&Annotation>;
    fn set_annotation(&mut self, a: Annotation);
}

pub fn annotate<T>(acc: &mut Option<Annotation>, _env: &mut Env<T>, node: NodeMut<T>) -> Result<()>
where
    T: AsExpr + Annotated,
{
    match node {
        NodeMut::Ann(a) => {
            let props = serde_yaml::from_str(format!("{{ {} }}", a.text).as_str())?;
            acc.replace(Annotation { props });
        }
        NodeMut::Decl(d) => {
            if let Some(ann) = acc.take() {
                d.expr.set_annotation(ann);
            }
        }
        _ => {}
    }
    Ok(())
}
