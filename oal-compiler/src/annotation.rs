use crate::errors::Result;
use crate::scope::Env;
use crate::transform::Transform;
use oal_syntax::ast::{Expr, Node};

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    pub props: serde_yaml::Mapping,
}

pub trait Annotated {
    fn annotation(&self) -> Option<&Annotation>;
    fn set_annotation(&mut self, a: Annotation);
}

pub fn annotate<T>(acc: &mut Option<Annotation>, env: &mut Env<T>, e: &mut T) -> Result<()>
where
    T: Node + Annotated,
{
    e.as_mut().transform(acc, env, annotate)?;
    match e.as_ref() {
        Expr::Ann(doc) => {
            let props = serde_yaml::from_str(format!("{{ {} }}", doc).as_str())?;
            acc.replace(Annotation { props });
        }
        _ => {
            if let Some(ann) = acc.take() {
                e.set_annotation(ann);
            }
        }
    }
    Ok(())
}
