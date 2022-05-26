use crate::errors::{Error, Kind, Result};
use crate::scope::Env;
use crate::tag::{Tag, Tagged};
use oal_syntax::ast::{
    Array, AsExpr, Expr, NodeRef, Object, Operator, Property, Relation, Transfer, Uri, UriSegment,
    VariadicOp,
};

trait TypeChecked {
    fn type_check(&self) -> Result<()> {
        Ok(())
    }
}

impl<T: AsExpr + Tagged> TypeChecked for VariadicOp<T> {
    fn type_check(&self) -> Result<()> {
        match self.op {
            Operator::Join => {
                if self.exprs.iter().all(|e| e.unwrap_tag() == Tag::Object) {
                    Ok(())
                } else {
                    Err(Error::new(Kind::InvalidTypes, "ill-formed join").with(self))
                }
            }
            Operator::Any | Operator::Sum => {
                if self.exprs.iter().all(|e| e.unwrap_tag().is_schema()) {
                    Ok(())
                } else {
                    Err(Error::new(Kind::InvalidTypes, "ill-formed alternative").with(self))
                }
            }
        }
    }
}

impl<T: AsExpr + Tagged> TypeChecked for Transfer<T> {
    fn type_check(&self) -> Result<()> {
        let domain_check = if let Some(domain) = &self.domain {
            domain.unwrap_tag().is_schema_like()
        } else {
            true
        };
        let range_check = self.range.unwrap_tag().is_schema_like();
        let params_check = self
            .params
            .as_ref()
            .map_or(true, |p| p.unwrap_tag() == Tag::Object);
        if domain_check && range_check && params_check {
            Ok(())
        } else {
            Err(Error::new(Kind::InvalidTypes, "ill-formed transfer").with(self))
        }
    }
}

impl<T: AsExpr + Tagged> TypeChecked for Relation<T> {
    fn type_check(&self) -> Result<()> {
        let uri_check = self.uri.unwrap_tag() == Tag::Uri;
        let xfers_check = self.xfers.iter().all(|x| x.unwrap_tag() == Tag::Transfer);
        if uri_check && xfers_check {
            Ok(())
        } else {
            Err(Error::new(Kind::InvalidTypes, "ill-formed relation").with(self))
        }
    }
}

impl<T: AsExpr + Tagged> TypeChecked for Uri<T> {
    fn type_check(&self) -> Result<()> {
        let vars_check = self.path.iter().all(|s| {
            if let UriSegment::Variable(var) = s {
                if let Expr::Property(prop) = var.as_node().as_expr() {
                    prop.val.unwrap_tag() == Tag::Primitive
                } else {
                    false
                }
            } else {
                true
            }
        });
        let params_check = self
            .params
            .as_ref()
            .map_or(true, |p| p.unwrap_tag() == Tag::Object);
        if vars_check && params_check {
            Ok(())
        } else {
            Err(Error::new(Kind::InvalidTypes, "ill-formed URI").with(self))
        }
    }
}

impl<T: AsExpr + Tagged> TypeChecked for Array<T> {
    fn type_check(&self) -> Result<()> {
        if self.item.unwrap_tag().is_schema() {
            Ok(())
        } else {
            Err(Error::new(Kind::InvalidTypes, "ill-formed array").with(self))
        }
    }
}

impl<T: AsExpr + Tagged> TypeChecked for Property<T> {
    fn type_check(&self) -> Result<()> {
        if self.val.unwrap_tag().is_schema() {
            Ok(())
        } else {
            Err(Error::new(Kind::InvalidTypes, "ill-formed property").with(self))
        }
    }
}

impl<T: AsExpr + Tagged> TypeChecked for Object<T> {
    fn type_check(&self) -> Result<()> {
        if self.props.iter().all(|p| p.unwrap_tag() == Tag::Property) {
            Ok(())
        } else {
            Err(Error::new(Kind::InvalidTypes, "ill-formed object").with(self))
        }
    }
}

pub fn type_check<T>(_acc: &mut (), _env: &mut Env<T>, node: NodeRef<T>) -> Result<()>
where
    T: AsExpr + Tagged,
{
    if let NodeRef::Expr(e) = node {
        match e.as_node().as_expr() {
            Expr::Op(op) => op.type_check(),
            Expr::Rel(rel) => rel.type_check(),
            Expr::Uri(uri) => uri.type_check(),
            Expr::Array(arr) => arr.type_check(),
            Expr::Property(prop) => prop.type_check(),
            Expr::Object(obj) => obj.type_check(),
            Expr::Xfer(xfer) => xfer.type_check(),
            _ => Ok(()),
        }
    } else {
        Ok(())
    }
}
