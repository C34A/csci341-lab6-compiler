use crate::expr::*;
use logos::Logos;


#[derive(Debug, Logos, PartialEq)]
pub enum Tok<'a> {
  #[regex("[0-9]+", |lex| lex.slice().parse())]
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

  #[token("=")]
  Equals,
  #[token(";")]
  Semicolon,

  #[token("let")]
  Let,

  #[token("(")]
  LParen,
  #[token(")")]
  RParen,

  #[token(",")]
  Comma,

  // this lets us not need lookahead which and is a bit of a hack but makes life easier
  #[token("set")]
  Set,

  #[regex(r"//.*[\n\r]", logos::skip)]
  #[regex(r"[ \t\n\r\f]+", logos::skip)]
  #[error]
  Error,
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
    match self.tokens.pop() {
      Some(Tok::Error) => {
        eprintln!("ERR: unrecognized token");
        self.pop()
      },
      other => other,
    }
  }

  fn peek(&mut self) -> Option<&Tok<'a>> {
    match self.tokens.last() {
      Some(Tok::Error) => {
        panic!("ERR: unrecognized token: {:?}", self.tokens.last().unwrap());
        // self.tokens.pop();
        // self.peek()
      },
      other => other,
    }
  }
}

pub fn parse(input: &str) -> Option<Vec<Stmt>> {
  let mut lex = Lex::new(input);

  // for t in lex.tokens.iter().rev() {
  //   println!("{:?}", t);
  // }

  let mut ret = vec![];

  loop {
    if let Some(s) = parse_decl(&mut lex) {
      ret.push(s);
      continue;
    } else if let Some(s) = parse_assign(&mut lex) {
      ret.push(s);
      continue;
    } else if let Some(s) = parse_expr_stmt(&mut lex) {
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

fn parse_decl<'a>(lex: &mut Lex) -> Option<Stmt> {
  match_tok(lex, Tok::Let)?;
  let name = if let Some(Tok::Ident(name)) = match_tok(lex, Tok::Ident("" as _)) {
    name.into()
  } else {
    eprintln!("ERR: identifier expected in declaration");
    synchronize(lex);
    return None;
  };

  let val = if let Some(_) = match_tok(lex, Tok::Equals) {
    if let Some(Tok::Lit(val)) = match_tok(lex, Tok::Lit(0)) {
      val
    } else {
      eprintln!("ERR: value expected in declaration (note: expressions are not supported here)");
      synchronize(lex);
      return None;
    }
  } else {0};

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
      // this is stupid but fine for now
      (Tok::Equals, Tok::Equals) => Some(lex.pop()?),
      (Tok::Ident(_), Tok::Ident(_)) => Some(lex.pop()?),
      (Tok::Let, Tok::Let) => Some(lex.pop()?),
      (Tok::Set, Tok::Set) => Some(lex.pop()?),
      (Tok::Lit(_), Tok::Lit(_)) => Some(lex.pop()?),
      (Tok::Minus, Tok::Minus) => Some(lex.pop()?),
      (Tok::Plus, Tok::Plus) => Some(lex.pop()?),
      (Tok::Semicolon, Tok::Semicolon) => Some(lex.pop()?),
      (Tok::Slash, Tok::Slash) => Some(lex.pop()?),
      (Tok::Star, Tok::Star) => Some(lex.pop()?),
      (Tok::LParen, Tok::LParen) => Some(lex.pop()?),
      (Tok::RParen, Tok::RParen) => Some(lex.pop()?),
      (Tok::Comma, Tok::Comma) => Some(lex.pop()?),
      _ => None,
    }
  } else {
    None
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

  let val = if let Some(e) = parse_sum(lex) {
    e
  } else {
    eprintln!("ERR: expression expected in assignment");
    synchronize(lex);
    return None;
  };

  if let Some(_) = match_tok(lex, Tok::Semicolon) {
    Some(Stmt::Assignment(name, val))
  } else {
    eprintln!("ERR: semicolon expected after assignment");
    synchronize(lex);
    None
  }
}

fn parse_expr_stmt<'a>(lex: &mut Lex) -> Option<Stmt> {
  let e = Stmt::ExprStmt(parse_sum(lex)?);
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
    _ => {
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
        return Some(Expr::Call(s, vec![]));
      } else if let Some(e) = parse_sum(lex) { // one or more params
        let mut params = vec![e];
        loop {
          // if matches!(lex.peek(), Some(Tok::RParen)) {
          //   break;
          // }
          println!("{:?}", lex.peek());
          match lex.peek() {
            Some(Tok::RParen) => break,
            Some(Tok::Comma) => (),
            _ => {
              eprintln!("ERR: expected comma between call parameters");
              return None;
            }
          }
          lex.pop(); // eat comma
          match parse_sum(lex) {
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

fn parse_term<'a>(lex: &mut Lex) -> Option<Expr>  {
  let first = parse_call(lex)?;
  let op = match lex.peek() {
    Some(Tok::Star) => {
      lex.pop()?; // eat op
      BinOp::Mul
    },
    Some(Tok::Slash) => {
      lex.pop()?; // eat op
      BinOp::Div
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