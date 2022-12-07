

#[derive(Debug)]
pub enum Stmt {
  ExprStmt(Expr),
  Decl(String, DeclInit),
  Assignment(String, Expr),
}


#[derive(Debug)]
pub enum DeclInit {
  Str(String),
  Int(i64),
}

#[derive(Debug)]
pub enum Expr {
  Lit(i64),
  String(String),
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
