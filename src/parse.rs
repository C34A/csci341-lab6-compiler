
use crate::expr::*;
use logos::{Logos, Lexer};


#[derive(Debug, Logos, PartialEq)]
pub enum Tok<'a> {
  #[regex("(-?)[0-9]+", |lex| lex.slice().parse())]
  Lit(i64),

  #[regex(r"[a-zA-Z_][a-zA-Z_\-0-9]*", |lex| lex.slice())]
  Ident(&'a str),

  #[token("+")]
  Plus,
  #[token("-")]
  Minus,
  #[token("*")]
  Star,
  #[token("/")]
  Slash,
  #[token("%")]
  Rem,
  #[token("!")]
  Bang,
  #[token("&")]
  Amp,
  #[token("~")]
  Tilde,

  #[token("=")]
  Equals,
  #[token(";")]
  Semicolon,

  #[token("==")]
  EqEq,
  #[token("<")]
  Less,
  #[token("<_")]
  LessUnsigned,
  #[token(">")]
  Greater,

  #[token("|")]
  Or,
  #[token("^")]
  Xor,

  #[token(">>_")]
  RShift,
  #[token(">>")]
  ARShift,
  #[token("<<")]
  LShift,

  #[token("let")]
  Let,

  #[token("(")]
  LParen,
  #[token(")")]
  RParen,

  #[token("{")]
  LBracket,
  #[token("}")]
  RBracket,
  #[token("if")]
  If,
  #[token("else")]
  Else,
  #[token("while")]
  While,

  #[token(",")]
  Comma,

  #[regex(r#""([^"\\]|\\t|\\r|\\n|\\")*""#, lex_str)
  ]
  String(&'a str),

  // this lets us not need lookahead which and is a bit of a hack but makes life easier
  #[token("set")]
  Set,

  #[regex(r"//.*[\n\r]", logos::skip)]
  #[regex(r"[ \t\n\r\f]+", logos::skip)]
  #[error]
  Error,
}

fn lex_str<'a>(lex: &mut Lexer<'a, Tok<'a>>) -> Option<&'a str> {
  Some(&lex.slice()[1..lex.slice().len()-1])
}

struct Lex<'a> {
  tokens: Vec<Tok<'a>>
}

impl<'a> Lex<'a> {
  fn new(input: &'a str) -> Self {
    let l = Tok::lexer(input);
    let mut v = Vec::from_iter(l.into_iter());
    v.reverse();
    Self {
      tokens: v
    }
  }

  fn pop(&mut self) -> Option<Tok<'a>> {
    let ret = 
        match self.tokens.pop() {
        Some(Tok::Error) => {
          eprintln!("ERR: unrecognized token");
          self.pop()
        },
        other => other,
      }
    ;
    ret
  }

  fn peek(&mut self) -> Option<&Tok<'a>> {
    match self.tokens.last() {
      Some(Tok::Error) => {
        panic!("ERR: unrecognized token: {:?}", self.tokens.last().unwrap());
      },
      other => other,
    }
  }

  fn push(&mut self, t: Tok<'a>) {
    self.tokens.push(t)
  }
}

pub fn parse(input: &str) -> Option<Block> {
  let mut lex = Lex::new(input);

  // useful for lexer debugging:
  // for t in lex.tokens.iter().rev() {
  //   println!("{:?}", t);
  // }

  parse_block(&mut lex)
}

fn parse_block<'a>(lex: &mut Lex) -> Option<Block> {
  let mut ret = vec![];

  loop { // todo: something more fault tolerant?
    if let Some(s) = parse_decl(lex) {
      ret.push(s);
      continue;
    } else if let Some(s) = parse_assign(lex) {
      ret.push(s);
      continue;
    } else if let Some(s) = parse_expr_stmt(lex) {
      ret.push(s);
      continue;
    } else if let Some(s) = parse_if_stmt(lex) {
      ret.push(s);
      continue;
    } else if let Some(s) = parse_while(lex) {
      ret.push(s);
      continue;
    } else {
      break;
    }
  }

  if ret.len() >= 1 {
    Some(ret)
  } else {
    None
  }
}

fn parse_if_stmt<'a>(lex: &mut Lex) -> Option<Stmt> {
  match lex.peek() {
    Some(Tok::If) => {lex.pop(); ()},
    _ => {
      return None
    },
  }
  let mut ok = true;
  let condition = parse_expr(lex);

  expect(lex, Tok::LBracket, "ERR: expected { after if")?;

  let true_block = parse_block(lex);
  ok = if let Some(_) = expect(lex, Tok::RBracket, "ERR: expected } after if 'true' block")
    { ok } else { false };

  let else_block = if let Some(_) = match_tok(lex, Tok::Else) {
    expect(lex, Tok::LBracket, "ERR: expected { after else")?;
    let b = parse_block(lex);
    expect(lex, Tok::RBracket, "ERR: expected } after else block")
      .map(|_| b.unwrap())
  } else { None };

  if ok {
    Some(Stmt::If(condition?, true_block?, else_block))
  } else {
    None
  }
}

fn parse_while<'a>(lex: &mut Lex) -> Option<Stmt> {
  match lex.peek() {
    Some(Tok::While) => {lex.pop(); ()},
    _ => {
      return None
    },
  }
  let condition = parse_expr(lex);

  expect(lex, Tok::LBracket, "ERR: expected { after while")?;

  let true_block = parse_block(lex);
  expect(lex, Tok::RBracket, "ERR: expected } after while body")?;

  Some(Stmt::While(condition?, true_block?))
}

fn parse_decl<'a>(lex: &mut Lex) -> Option<Stmt> {
  match_tok(lex, Tok::Let)?;
  let name = if let Some(Tok::Ident(name)) = match_tok(lex, Tok::Ident("" as _)) {
    name.into()
  } else {
    eprintln!("ERR: identifier expected in declaration");
    synchronize(lex);
    return None;
  };

  // parse assignment, or fill in 0 otherwise
  let val = if let Some(_) = match_tok(lex, Tok::Equals) {
    match lex.pop() {
      Some(Tok::Lit(val)) => {
        DeclInit::Int(val)
      },
      Some(Tok::String(s)) => {
        DeclInit::Str(s.into())
      },
      _ => {
        eprintln!("ERR: value expected in declaration (note: expressions are not supported here)");
        synchronize(lex);
        return None;
      }
    }
  } else { DeclInit::Int(0) };

  if let Some(_) = match_tok(lex, Tok::Semicolon) {
    Some(Stmt::Decl(name, val))
  } else {
    eprintln!("ERR: semicolon expected after declaration");
    synchronize(lex);
    None
  }
}

// todo: put this into impl Lex
fn match_tok<'a>(lex: &mut Lex<'a>, expected: Tok) -> Option<Tok<'a>> {
  if let Some(t) = lex.peek() {
    match (t, expected) {
      // this is stupid but fine for now (ie works)
      (Tok::Lit(_), Tok::Lit(_)) => Some(lex.pop()?),
      (Tok::Ident(_), Tok::Ident(_)) => Some(lex.pop()?),
      (Tok::Plus, Tok::Plus) => Some(lex.pop()?),
      (Tok::Minus, Tok::Minus) => Some(lex.pop()?),
      (Tok::Star, Tok::Star) => Some(lex.pop()?),
      (Tok::Slash, Tok::Slash) => Some(lex.pop()?),
      (Tok::Rem, Tok::Rem) => Some(lex.pop()?),
      (Tok::Bang, Tok::Bang) => Some(lex.pop()?),
      (Tok::Amp, Tok::Amp) => Some(lex.pop()?),
      (Tok::Tilde, Tok::Tilde) => Some(lex.pop()?),
      (Tok::Equals, Tok::Equals) => Some(lex.pop()?),
      (Tok::Semicolon, Tok::Semicolon) => Some(lex.pop()?),
      (Tok::EqEq, Tok::EqEq) => Some(lex.pop()?),
      (Tok::Less, Tok::Less) => Some(lex.pop()?),
      (Tok::LessUnsigned, Tok::LessUnsigned) => Some(lex.pop()?),
      (Tok::Greater, Tok::Greater) => Some(lex.pop()?),
      (Tok::Or, Tok::Or) => Some(lex.pop()?),
      (Tok::Xor, Tok::Xor) => Some(lex.pop()?),
      (Tok::RShift, Tok::RShift) => Some(lex.pop()?),
      (Tok::ARShift, Tok::ARShift) => Some(lex.pop()?),
      (Tok::LShift, Tok::LShift) => Some(lex.pop()?),
      (Tok::Let, Tok::Let) => Some(lex.pop()?),
      (Tok::LParen, Tok::LParen) => Some(lex.pop()?),
      (Tok::RParen, Tok::RParen) => Some(lex.pop()?),
      (Tok::LBracket, Tok::LBracket) => Some(lex.pop()?),
      (Tok::RBracket, Tok::RBracket) => Some(lex.pop()?),
      (Tok::If, Tok::If) => Some(lex.pop()?),
      (Tok::Else, Tok::Else) => Some(lex.pop()?),
      (Tok::While, Tok::While) => Some(lex.pop()?),
      (Tok::Comma, Tok::Comma) => Some(lex.pop()?),
      (Tok::String(_), Tok::String(_)) => Some(lex.pop()?),
      (Tok::Set, Tok::Set) => Some(lex.pop()?),
      (_, _) => {
        // println!("expected {:?} got {:?}", e, t);
        None
      },
    }
  } else {
    None
  }
}

fn expect<'a>(lex: &mut Lex<'a>, expected: Tok, message: &'static str) -> Option<Tok<'a>> {
  match match_tok(lex, expected) {
    Some(t) => Some(t),
    None => {
      eprintln!("{}", message);
      None
    }
  }
}

fn synchronize<'a>(lex: &mut Lex) {
  while let Some(t) = lex.pop() {
    if t == Tok::Semicolon {
      return;
    }
  }
}

fn parse_assign<'a>(lex: &mut Lex) -> Option<Stmt> {
  match_tok(lex, Tok::Set)?;

  let name = if let Some(Tok::Ident(name)) = match_tok(lex, Tok::Ident("" as _)) {
    name.into()
  } else {
    eprintln!("ERR: expected identifier after 'set'.");
    synchronize(lex);
    return None;
  };

  if let None = match_tok(lex, Tok::Equals) {
    eprintln!("ERR: expected '=' after set identifier.");
    synchronize(lex);
    return None;
  }

  let val = if let Some(e) = parse_expr(lex) {
    e
  } else {
    eprintln!("ERR: expression expected in assignment");
    synchronize(lex);
    return None;
  };

  if let Some(_) = match_tok(lex, Tok::Semicolon) {
    Some(Stmt::Assignment(name, val))
  } else {
    eprintln!("ERR: semicolon expected after assignment, got {:?}", lex.peek());
    synchronize(lex);
    None
  }
}

fn parse_expr_stmt<'a>(lex: &mut Lex) -> Option<Stmt> {
  let e = Stmt::ExprStmt(parse_expr(lex)?);
  if let Some(_) = match_tok(lex, Tok::Semicolon) {
    Some(e)
  } else {
    eprintln!("ERR: syntax error - semicolon expected in expression statement.");
    synchronize(lex);
    None
  }
}

fn parse_atom<'a>(lex: &mut Lex) -> Option<Expr> {
  match lex.pop() {
    Some(Tok::Ident(i)) => {
      Some(Expr::Ident(i.into()))
    },
    Some(Tok::Lit(l)) => {
      Some(Expr::Lit(l))
    },
    Some(Tok::String(s)) => {
      Some(Expr::String(s.into()))
    },
    Some(Tok::LParen) => {
      let ret = parse_expr(lex);
      match lex.peek() {
        Some(Tok::RParen) => {
          lex.pop(); // eat )
          ret
        },
        _ => {
          eprintln!("ERR: expected )");
          None
        }
      }
    }
    other => {
      other.map(|tok| lex.push(tok)); // un-eat token if it isnt valid
      None
    },
  }
}

fn parse_call<'a>(lex: &mut Lex) -> Option<Expr> {
  let first = parse_atom(lex)?;
  match (first, lex.peek()) {
    (Expr::Ident(s), Some(Tok::LParen)) => { // valid call
      lex.pop(); // eat (
      if let Some(Tok::RParen) = lex.peek() { // no params
        lex.pop(); // eat )
        return Some(Expr::Call(s, vec![]));
      } else if let Some(e) = parse_expr(lex) { // one or more params
        let mut params = vec![e];
        loop {
          // if matches!(lex.peek(), Some(Tok::RParen)) {
          //   break;
          // }
          match lex.peek() {
            Some(Tok::RParen) => break,
            Some(Tok::Comma) => (),
            _ => {
              eprintln!("ERR: expected comma between call parameters");
              return None;
            }
          }
          lex.pop(); // eat comma
          match parse_expr(lex) {
            Some(e) => params.push(e),
            None => return None, // should this report an error?
          }
        }
        lex.pop(); // eat )
        return Some(Expr::Call(s, params));
      } else { // invalid
        eprintln!("ERR: expected ')' or expression in function call");
        return None;
      }
    },
    (atom, _) => Some(atom) // something else
  }
}

fn parse_unary<'a>(lex: &mut Lex) -> Option<Expr> {
  let operator = match lex.peek() {
    Some(Tok::Amp) => UnaryOp::Addr,
    Some(Tok::Star) => UnaryOp::Deref,
    Some(Tok::Minus) => UnaryOp::Neg,
    Some(Tok::Tilde) => UnaryOp::Not,
    Some(Tok::Bang) => UnaryOp::BoolNot,
    _ => {
      return parse_call(lex)
    }
  };
  lex.pop(); // eat operator

  let operand = parse_unary(lex)?;
  Some(Expr::Unary(operator, Box::new(operand)))
}

fn parse_term<'a>(lex: &mut Lex) -> Option<Expr>  {
  let first = parse_unary(lex)?;
  let op = match lex.peek() {
    Some(Tok::Star) => {
      lex.pop()?; // eat op
      BinOp::Mul
    },
    Some(Tok::Slash) => {
      lex.pop()?; // eat op
      BinOp::Div
    },
    Some(Tok::Rem) => {
      lex.pop()?; // eat op
      BinOp::Rem
    },
    _ => return Some(first),
  };
  let second = parse_term(lex)?;
  Some(Expr::Bin(Box::new(first), op, Box::new(second)))
}

fn parse_sum<'a>(lex: &mut Lex) -> Option<Expr> {
  let first = parse_term(lex)?;
  let op = match lex.peek() {
    Some(Tok::Plus) => {
      lex.pop()?; // eat op
      BinOp::Add
    },
    Some(Tok::Minus) => {
      lex.pop()?; // eat op
      BinOp::Sub
    },
    _ => return Some(first),
  };
  let second = parse_sum(lex)?;
  Some(Expr::Bin(Box::new(first), op, Box::new(second)))
}

fn parse_shift<'a>(lex: &mut Lex) -> Option<Expr> {
  let first = parse_sum(lex)?;
  let op = match lex.peek() {
    Some(Tok::RShift) => {
      lex.pop()?; // eat op
      BinOp::Srl
    },
    Some(Tok::ARShift) => {
      lex.pop()?; // eat op
      BinOp::Sra
    },
    Some(Tok::LShift) => {
      lex.pop()?; // eat op
      BinOp::Sll
    },
    _ => return Some(first),
  };
  let second = parse_shift(lex)?;
  Some(Expr::Bin(Box::new(first), op, Box::new(second)))
}

fn parse_less<'a>(lex: &mut Lex) -> Option<Expr> {
  let first = parse_shift(lex)?;
  let op = match lex.peek() {
    Some(Tok::Less) => {
      lex.pop()?; // eat op
      BinOp::Less
    },
    Some(Tok::LessUnsigned) => {
      lex.pop()?; // eat op
      BinOp::LessUnsigned
    },
    Some(Tok::Greater) => {
      lex.pop()?; // eat op
      BinOp::Greater
    },
    _ => return Some(first),
  };
  let second = parse_less(lex)?;
  Some(Expr::Bin(Box::new(first), op, Box::new(second)))
}

fn parse_equality<'a>(lex: &mut Lex) -> Option<Expr> {
  let first = parse_less(lex)?;
  let op = match lex.peek() {
    Some(Tok::EqEq) => {
      lex.pop()?; // eat op
      BinOp::TestEq
    },
    _ => return Some(first),
  };
  let second = parse_equality(lex)?;
  Some(Expr::Bin(Box::new(first), op, Box::new(second)))
}

fn parse_and<'a>(lex: &mut Lex) -> Option<Expr> {
  let first = parse_equality(lex)?;
  let op = match lex.peek() {
    Some(Tok::Amp) => {
      lex.pop()?; // eat op
      BinOp::And
    },
    _ => return Some(first),
  };
  let second = parse_and(lex)?;
  Some(Expr::Bin(Box::new(first), op, Box::new(second)))
}

fn parse_xor<'a>(lex: &mut Lex) -> Option<Expr> {
  let first = parse_and(lex)?;
  let op = match lex.peek() {
    Some(Tok::Xor) => {
      lex.pop()?; // eat op
      BinOp::Xor
    },
    _ => return Some(first),
  };
  let second = parse_xor(lex)?;
  Some(Expr::Bin(Box::new(first), op, Box::new(second)))
}

fn parse_or<'a>(lex: &mut Lex) -> Option<Expr> {
  let first = parse_xor(lex)?;
  let op = match lex.peek() {
    Some(Tok::Or) => {
      lex.pop()?; // eat op
      BinOp::Or
    },
    _ => return Some(first),
  };
  let second = parse_or(lex)?;
  Some(Expr::Bin(Box::new(first), op, Box::new(second)))
}

fn parse_expr<'a>(lex: &mut Lex) -> Option<Expr> {
  parse_or(lex)
}
