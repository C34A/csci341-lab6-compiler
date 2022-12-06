

#[derive(Debug)]
pub enum Stmt {
  ExprStmt(Expr),
  Decl(String, i64),
  Assignment(String, Expr),
}

#[derive(Debug)]
pub enum Expr {
  Lit(i64),
  Ident(String),
  Bin(Box<Expr>, BinOp, Box<Expr>),
  Call(String, Vec<Expr>)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinOp {
  Add,
  Sub,
  Mul,
  Div,
}
