use std::iter::Peekable;

use crate::error::EvalErr;
use crate::lexer::{Token, TokenStream};
use crate::primitives::pair::Pair;
use crate::print::Printable;
use crate::procedure::Proc;
use crate::special_form::{And, Assignment, Define, If, Lambda, Or};
use crate::utils::{GetVals, ToExpr};

// We treat any list that is expected to be evaluated as a procedure during parsing as a vector
// of expressions rather than a proper list of pairs to simplify and reduce the cost of the parsing process.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    List(Vec<Expr>),
    Atom(Token),
    Proc(Proc),
    Dotted(Pair),
    If(Box<If>),
    Define(Box<Define>),
    Lambda(Box<Lambda>),
    Assignment(Box<Assignment>),
    And(And),
    Or(Or),
    Quoted(Box<Expr>),
    EmptyList,
}

impl Expr {
    fn as_list(self) -> Result<Expr, EvalErr> {
        Ok(Expr::List(vec![self]))
    }
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
            let expr = self.parse_from_token()?;
            self.parsed_exprs.push(expr)
        }
        Ok(self.parsed_exprs)
    }

    fn parse_from_token(&mut self) -> Result<Expr, EvalErr> {
        match self.next_or_err(EvalErr::UnexpectedEnd)? {
            Token::LParen => match self.peek_or_err(EvalErr::UnexpectedEnd)? {
                Token::If => {
                    self.tokens.next();
                    self.parse_if()?.as_list()
                }
                Token::Lambda => {
                    self.tokens.next();
                    self.parse_lambda()?.as_list()
                }
                Token::Define => {
                    self.tokens.next();
                    self.parse_define()?.as_list()
                }
                Token::Assignment => {
                    self.tokens.next();
                    self.parse_assignment()?.as_list()
                }
                Token::And => {
                    self.tokens.next();
                    self.parse_and()?.as_list()
                }
                Token::Or => {
                    self.tokens.next();
                    self.parse_or()?.as_list()
                }
                Token::QuoteProc => {
                    self.tokens.next();
                    let quoted = self.parse_quote();
                    self.next_or_err(EvalErr::UnexpectedEnd)?; // consume remaining paren
                    quoted
                }
                _ => self.parse_inner_list(),
            },
            Token::QuoteTick => self.parse_quote(),
            x @ Token::Number(_)
            | x @ Token::Str(_)
            | x @ Token::Boolean(_)
            | x @ Token::Symbol(_) => Ok(Expr::Atom(x)),
            // TODO: we may want to handle below as a special case? seems fine with a
            // generic UnexpectedToken error though.
            // p @ Token::RParen => Err(EvalErr::UnexpectedToken(p.printable())),
            t => Err(EvalErr::UnexpectedToken(t.printable())),
        }
    }

    fn parse_inner_list(&mut self) -> Result<Expr, EvalErr> {
        let mut parsed_exprs: Vec<Expr> = vec![];
        while let Some(t) = self.tokens.peek() {
            if let Ok(Token::RParen) = t {
                self.tokens.next();
                match parsed_exprs.len() {
                    0 => return Ok(Expr::EmptyList),
                    _ => return Ok(Expr::List(parsed_exprs)),
                }
            } else {
                parsed_exprs.push(self.parse_from_token()?)
            }
        }
        Err(EvalErr::UnexpectedEnd)
    }

    fn parse_if(&mut self) -> Result<Expr, EvalErr> {
        let arg_err = || {
            EvalErr::InvalidArgs("'if' expression. expected condition, predicate, and consequence")
        };
        match self.parse_inner_list()? {
            Expr::List(rest) => {
                let (p, c, a) = rest.into_iter().get_three_or_else(arg_err)?;
                Ok(If::new(p, c, a).to_expr())
            }
            Expr::EmptyList => Err(arg_err()),
            t => Err(EvalErr::UnexpectedToken(t.printable())),
        }
    }

    fn parse_lambda(&mut self) -> Result<Expr, EvalErr> {
        let arg_err = || EvalErr::InvalidArgs("'lambda' expression. expected parameters and body");
        match self.parse_inner_list()? {
            Expr::List(rest) => {
                let (first, rest) = rest.into_iter().get_one_and_rest_or_else(arg_err)?;
                Ok(Lambda::new(first, rest.collect()).to_expr())
            }
            Expr::EmptyList => Err(arg_err()),
            t => Err(EvalErr::UnexpectedToken(t.printable())),
        }
    }

    fn parse_define(&mut self) -> Result<Expr, EvalErr> {
        let arg_err = || EvalErr::InvalidArgs("'define' expression. expected identifier and value");
        match self.parse_inner_list()? {
            Expr::List(rest) => {
                let (first, rest) = rest.into_iter().get_one_and_rest_or_else(arg_err)?;
                Ok(Define::new(first, rest.collect()).to_expr())
            }
            Expr::EmptyList => Err(arg_err()),
            t => Err(EvalErr::UnexpectedToken(t.printable())),
        }
    }

    fn parse_assignment(&mut self) -> Result<Expr, EvalErr> {
        let arg_err = || EvalErr::InvalidArgs("'set!' expression. expected identifier and value");
        match self.parse_inner_list()? {
            Expr::List(rest) => {
                let (first, rest) = rest.into_iter().get_one_and_rest_or_else(arg_err)?;
                Ok(Assignment::new(first, rest.collect()).to_expr())
            }
            Expr::EmptyList => Err(arg_err()),
            t => Err(EvalErr::UnexpectedToken(t.printable())),
        }
    }

    fn parse_and(&mut self) -> Result<Expr, EvalErr> {
        let arg_err = || EvalErr::InvalidArgs("'and' expression. expected arguments");
        match self.parse_inner_list()? {
            Expr::List(rest) => Ok(And::new(rest).to_expr()),
            Expr::EmptyList => Err(arg_err()),
            t => Err(EvalErr::UnexpectedToken(t.printable())),
        }
    }

    fn parse_or(&mut self) -> Result<Expr, EvalErr> {
        let arg_err = || EvalErr::InvalidArgs("'or' expression. expected arguments");
        match self.parse_inner_list()? {
            Expr::List(rest) => Ok(Or::new(rest).to_expr()),
            Expr::EmptyList => Err(arg_err()),
            t => Err(EvalErr::UnexpectedToken(t.printable())),
        }
    }

    // We treat a quoted expression as a normal expression behind an extra indrection, with the
    // addtional major differnce being we parse lists as pairs instead of vectors. this way they
    // can be accessed at runtime rather than evaluated as procedures.
    //
    // TODO: we might consider parsing this in the same way we do non-quoted tokens, but this is a
    // bit more lenient as any token is valid (besides the hanging closing paren).
    fn parse_quote(&mut self) -> Result<Expr, EvalErr> {
        let res = match self.next_or_err(EvalErr::UnexpectedEnd)? {
            Token::LParen => {
                let res = self.parse_inner_quote()?;
                self.tokens.next(); // consume remaining paren
                Ok(res)
            }
            t @ Token::Assignment
            | t @ Token::Lambda
            | t @ Token::Define
            | t @ Token::If
            | t @ Token::And
            | t @ Token::QuoteTick
            | t @ Token::QuoteProc
            | t @ Token::Or => Ok(t.printable().to_expr()),
            // TODO: I'm pretty sure we hanlde nested quoted exprs in this way but double check
            // Token::QuoteTick => self.parse_quote(),
            // Token::QuoteProc => self.parse_quote(),
            x @ Token::Number(_)
            | x @ Token::Str(_)
            | x @ Token::Boolean(_)
            | x @ Token::Symbol(_) => Ok(Expr::Atom(x)),
            p @ Token::RParen => Err(EvalErr::UnexpectedToken(p.printable())),
        }?;

        Ok(Expr::Quoted(Box::new(res)))
    }

    fn parse_inner_quote(&mut self) -> Result<Expr, EvalErr> {
        match self.tokens.peek() {
            Some(t) => match t {
                Ok(Token::RParen) => Ok(Expr::EmptyList),
                Ok(_) => {
                    let current = self.parse_quote()?;
                    let next = self.parse_inner_quote()?;
                    Ok(Expr::Dotted(Pair::new(current, next)))
                }
                Err(x) => Err(x.to_owned()),
            },
            None => Err(EvalErr::UnexpectedEnd),
        }
    }

    fn next_or_err(&mut self, err: EvalErr) -> Result<Token, EvalErr> {
        self.tokens.next().map_or_else(|| Err(err), Ok)?
    }

    fn peek_or_err(&mut self, err: EvalErr) -> Result<&Token, EvalErr> {
        self.tokens.peek().map_or_else(
            || Err(err),
            |t| match t {
                Ok(t) => Ok(t),
                Err(e) => Err(e.clone()),
            },
        )
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
    fn quoted() {
        let scm = "'(+ 1)";
        let res: Vec<Expr> = vec![Expr::Quoted(Box::new(Expr::Dotted(Pair::new(
            Expr::Quoted(Box::new(Expr::Atom(Token::Symbol("+".to_string())))),
            Expr::Dotted(Pair::new(
                Expr::Quoted(Box::new(Expr::Atom(Token::Number(1.0)))),
                Expr::EmptyList,
            )),
        ))))];
        let exprs = Parser::new(TokenStream::new(scm)).parse().unwrap();
        assert_eq!(res, exprs);
    }

    #[test]
    fn quoted_fn() {
        let scm = "(quote (+ 1))";
        let res: Vec<Expr> = vec![Expr::Quoted(Box::new(Expr::Dotted(Pair::new(
            Expr::Quoted(Box::new(Expr::Atom(Token::Symbol("+".to_string())))),
            Expr::Dotted(Pair::new(
                Expr::Quoted(Box::new(Expr::Atom(Token::Number(1.0)))),
                Expr::EmptyList,
            )),
        ))))];
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
