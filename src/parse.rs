use std::iter::Peekable;

use crate::expr::*;
use logos::Logos;


#[derive(Debug, Logos, PartialEq)]
pub enum Tok<'a> {
  #[regex("[0-9]+", |lex| lex.slice().parse())]
  Lit(i64),

  #[regex("[a-zA-Z]+", |lex| lex.slice())]
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

  // this lets us not need lookahead which and is a bit of a hack but makes life easier
  #[token("set")]
  Set,

  #[regex(r"//.*[\n\r]", logos::skip)]
  #[regex(r"[ \t\n\r\f]+", logos::skip)]
  #[error]
  Error,
}

pub fn parse(input: &str) -> Option<Vec<Stmt>> {
  let mut lex = Tok::lexer(input).peekable();

  let mut ret = vec![];

  loop {
    if let Some(s) = parse_decl(&mut lex) {
      ret.push(s);
      continue;
    } else if let Some(s) = parse_assign(&mut lex) {
      ret.push(s);
      continue;
    }else if let Some(s) = parse_expr_stmt(&mut lex) {
      ret.push(s);
      continue;
    } else {
      break;
    }
  }

  if ret.len() > 1 {
    Some(ret)
  } else {
    None
  }
}

fn parse_decl<'a>(lex: &mut Peekable<impl Iterator<Item = Tok<'a>>>) -> Option<Stmt> {
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

fn match_tok<'a>(lex: &mut Peekable<impl Iterator<Item = Tok<'a>>>, expected: Tok) -> Option<Tok<'a>> {
  if let Some(t) = lex.peek() {
    match (t, expected) {
      // this is stupid but fine for now
      (Tok::Equals, Tok::Equals) => Some(lex.next()?),
      (Tok::Ident(_), Tok::Ident(_)) => Some(lex.next()?),
      (Tok::Let, Tok::Let) => Some(lex.next()?),
      (Tok::Set, Tok::Set) => Some(lex.next()?),
      (Tok::Lit(_), Tok::Lit(_)) => Some(lex.next()?),
      (Tok::Minus, Tok::Minus) => Some(lex.next()?),
      (Tok::Plus, Tok::Plus) => Some(lex.next()?),
      (Tok::Semicolon, Tok::Semicolon) => Some(lex.next()?),
      (Tok::Slash, Tok::Slash) => Some(lex.next()?),
      (Tok::Star, Tok::Star) => Some(lex.next()?),
      _ => None,
    }
  } else {
    None
  }
}

fn synchronize<'a>(lex: &mut Peekable<impl Iterator<Item = Tok<'a>>>) {
  while let Some(t) = lex.next() {
    if t == Tok::Semicolon {
      return;
    }
  }
}

fn parse_assign<'a>(lex: &mut Peekable<impl Iterator<Item = Tok<'a>>>) -> Option<Stmt> {
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

fn parse_expr_stmt<'a>(lex: &mut Peekable<impl Iterator<Item = Tok<'a>>>) -> Option<Stmt> {
  let e = Stmt::ExprStmt(parse_sum(lex)?);
  if let Some(_) = match_tok(lex, Tok::Semicolon) {
    Some(e)
  } else {
    eprintln!("ERR: syntax error.");
    synchronize(lex);
    None
  }
}

fn parse_atom<'a>(lex: &mut Peekable<impl Iterator<Item = Tok<'a>>>) -> Option<Expr> {
  match lex.next() {
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

fn parse_term<'a>(lex: &mut Peekable<impl Iterator<Item = Tok<'a>>>) -> Option<Expr>  {
  let first = parse_atom(lex)?;
  let op = match lex.peek() {
    Some(Tok::Star) => {
      lex.next()?; // eat op
      BinOp::Mul
    },
    Some(Tok::Slash) => {
      lex.next()?; // eat op
      BinOp::Div
    },
    _ => return Some(first),
  };
  let second = parse_term(lex)?;
  Some(Expr::Bin(Box::new(first), op, Box::new(second)))
}

fn parse_sum<'a>(lex: &mut Peekable<impl Iterator<Item = Tok<'a>>>) -> Option<Expr> {
  let first = parse_term(lex)?;
  let op = match lex.peek() {
    Some(Tok::Plus) => {
      lex.next()?; // eat op
      BinOp::Add
    },
    Some(Tok::Minus) => {
      lex.next()?; // eat op
      BinOp::Sub
    },
    _ => return Some(first),
  };
  let second = parse_sum(lex)?;
  Some(Expr::Bin(Box::new(first), op, Box::new(second)))
}