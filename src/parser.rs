use std::iter::Peekable;

use crate::error::EvalErr;
use crate::lexer::{Token, TokenStream};
use crate::primitives::pair::Pair;
use crate::procedure::Proc;
use crate::special_form::{Define, If, Lambda};
use crate::utils::{GetVals, ToExpr};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    List(Vec<Expr>),
    Atom(Token),
    Proc(Proc),
    Dotted(Pair),
    Define(Box<Define>),
    Lambda(Box<Lambda>),
    If(Box<If>),
    EmptyList,
    // Quoted(Box<Expr>),
}

pub struct Parser<'a> {
    tokens: Peekable<TokenStream<'a>>,
    parsed_exprs: Vec<Expr>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: TokenStream<'a>) -> Self {
        Parser {
            tokens: tokens.peekable(),
            parsed_exprs: vec![],
        }
    }

    pub fn parse(mut self) -> Result<Vec<Expr>, EvalErr> {
        while self.tokens.peek().is_some() {
            let expr = self.read_from_tokens()?;
            self.parsed_exprs.push(expr)
        }
        Ok(self.parsed_exprs)
    }

    fn read_from_tokens(&mut self) -> Result<Expr, EvalErr> {
        if let Some(token) = self.tokens.next() {
            match token? {
                Token::LParen => {
                    let res = self.parse_out_list()?;
                    self.tokens.next(); // consume remaining paren
                    Ok(res)
                }
                Token::If => self.parse_if(),
                Token::Lambda => self.parse_lambda(),
                Token::Define => self.parse_define(),
                x @ Token::Number(_)
                | x @ Token::Str(_)
                | x @ Token::Boolean(_)
                | x @ Token::Symbol(_) => Ok(Expr::Atom(x)),
                Token::RParen => Err(EvalErr::UnexpectedToken("unexpected )".to_string())),
            }
        } else {
            Err(EvalErr::UnexpectedEnd)
        }
    }

    fn parse_out_list(&mut self) -> Result<Expr, EvalErr> {
        let mut parsed_exprs: Vec<Expr> = vec![];
        while let Some(t) = self.tokens.peek() {
            if let Ok(Token::RParen) = t {
                match parsed_exprs.len() {
                    0 => return Ok(Expr::EmptyList),
                    _ => return Ok(Expr::List(parsed_exprs)),
                }
            } else {
                parsed_exprs.push(self.read_from_tokens()?)
            }
        }
        Err(EvalErr::UnexpectedEnd)
    }

    fn parse_if(&mut self) -> Result<Expr, EvalErr> {
        match self.parse_out_list()? {
            Expr::List(rest) => {
                let (p, c, a) = rest.into_iter().get_three()?;
                Ok(If::new(p, c, a).to_expr())
            }
            _ => Err(EvalErr::UnexpectedToken("expected list".to_string())),
        }
    }

    fn parse_lambda(&mut self) -> Result<Expr, EvalErr> {
        match self.parse_out_list()? {
            Expr::List(rest) => {
                let (first, rest) = rest.into_iter().get_one_and_rest()?;
                Ok(Lambda::new(first, rest.collect()).to_expr())
            }
            _ => Err(EvalErr::UnexpectedToken("expected list".to_string())),
        }
    }

    fn parse_define(&mut self) -> Result<Expr, EvalErr> {
        match self.parse_out_list()? {
            Expr::List(rest) => {
                let (first, rest) = rest.into_iter().get_one_and_rest()?;
                Ok(Define::new(first, rest.collect()).to_expr())
            }
            _ => Err(EvalErr::UnexpectedToken("expected list".to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_parse() {
        let scm = "1 (+ 1 (+ 1 2))";
        let res: Vec<Expr> = vec![
            Expr::Atom(Token::Number(1.0)),
            Expr::List(vec![
                Expr::Atom(Token::Symbol("+".to_string())),
                Expr::Atom(Token::Number(1.0)),
                Expr::List(vec![
                    Expr::Atom(Token::Symbol("+".to_string())),
                    Expr::Atom(Token::Number(1.0)),
                    Expr::Atom(Token::Number(2.0)),
                ]),
            ]),
        ];
        let exprs = Parser::new(TokenStream::new(scm)).parse().unwrap();
        assert_eq!(res, exprs);
    }

    #[test]
    #[should_panic]
    fn extra_paren() {
        let scm = "(+ 1 2) (1))";
        Parser::new(TokenStream::new(scm)).parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn opening_rparen() {
        let scm = ")(yo)";
        Parser::new(TokenStream::new(scm)).parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn no_close() {
        let scm = "(+ 1 (1)";
        Parser::new(TokenStream::new(scm)).parse().unwrap();
    }
}
