
pub type Block = Vec<Stmt>;

#[derive(Debug)]
pub enum Stmt {
  ExprStmt(Expr),
  Decl(String, DeclInit),
  Assignment(String, Expr),
  If(Expr, Block, Option<Block>),
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
  Call(String, Vec<Expr>),
  Unary(UnaryOp, Box<Expr>),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinOp {
  Add,
  Sub,
  Mul,
  Div,
  Rem,
  Srl,
  Sra,
  Sll,
  And,
  Or,
  Xor,
  Less,
  LessUnsigned,
  Greater,
  TestEq,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UnaryOp {
  Deref,
  Addr,
  Neg,
  Not,
  BoolNot,
}
