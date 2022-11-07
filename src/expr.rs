

#[derive(Debug)]
pub enum Expr {
  Lit(i64),
  Ident(String),
  Bin(Box<Expr>, BinOp, Box<Expr>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinOp {
  Add,
  Sub,
  Mul,
  Div,
}
