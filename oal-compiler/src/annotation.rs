use crate::errors::Result;
use crate::scope::Env;
use oal_syntax::ast::{AsExpr, Expr, NodeMut};

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    pub props: serde_yaml::Mapping,
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
        NodeMut::Expr(e) => match e.as_ref() {
            Expr::Ann(doc) => {
                let props = serde_yaml::from_str(format!("{{ {} }}", doc).as_str())?;
                acc.replace(Annotation { props });
            }
            _ => {
                if let Some(ann) = acc.take() {
                    e.set_annotation(ann);
                }
            }
        },
        _ => {}
    }
    Ok(())
}
