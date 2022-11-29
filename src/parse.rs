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

  #[regex(r"[ \t\n\r\f]+", logos::skip)]
  #[error]
  Error,
}

pub fn parse(input: &str) -> Option<Expr> {
  let mut lex = Tok::lexer(input).peekable();

  parse_sum(&mut lex)
}

fn parse_atom<'a>(lex: &mut Peekable<impl Iterator<Item = Tok<'a>>>) -> Option<Expr> {
  match lex.next() {
    Some(Tok::Ident(i)) => {
      Some(Expr::Ident(i.into()))
    },
    Some(Tok::Lit(l)) => {
      Some(Expr::Lit(l))
    },
    _ => None,
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