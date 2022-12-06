use crate::expr::*;

pub struct TExpr {
  pub ty: Ty,
  pub exp: TExprKind,
}

pub enum TExprKind {
  Lit(i64),
  Ident(String),
  Bin(Box<TExpr>, BinOp, Box<TExpr>),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Ty {
  I32,
  U32,
}

pub fn typecheck(e: &Expr, errs: &mut Vec<String>) -> Option<TExpr> {
    
  match e {
    Expr::Lit(val) => {
      Some(TExpr{
        ty: Ty::I32,
        exp: TExprKind::Lit(*val),
      })
    },
    Expr::Ident(_name) => unimplemented!(),
    Expr::Bin(left, op, right) => {
      let left = typecheck(left, errs);
      let right = typecheck(right, errs);

      let left = left?;
      let right = right?;

      let new_ty = if left.ty == right.ty { left.ty } else {
        errs.push(format!("types for binary operator do not match: got {:?} and {:?}", left.ty, right.ty));
        return None;
      };

      Some(TExpr {
        ty: new_ty,
        exp: TExprKind::Bin(Box::new(left), *op, Box::new(right))
      })
    }
    Expr::Call(_, _) => todo!(),
  }
  
}
