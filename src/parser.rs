use std::iter::Peekable;
use std::rc::Rc;
use std::vec::IntoIter;

use crate::error::EvalErr;
use crate::lexer::Token;
use crate::primitives::pair::Pair;
use crate::print::Printable;
use crate::procedure::Proc;
use crate::special_form::{And, Assignment, Begin, Cond, Define, If, Lambda, Let, Or};
use crate::utils::{GetVals, ToExpr};

// We treat any list that is expected to be evaluated as a procedure during parsing as a vector
// of expressions rather than a proper list of pairs to simplify and reduce the cost of the parsing process.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // evaluable list
    // is it possible to just tranmute this into Proc?
    List(Vec<Expr>),

    // since these clone on getting values from env,
    // we want to allow multiple ownership with Rc to prevent deep cloning lists etc
    Dotted(Rc<Pair>), //TODO: Maybe Rc -> Rc<RefCell>? unsafe mutation seems to be ok for now...
    Proc(Proc),       //TODO: Proc -> Rc<Proc>

    Atom(Token),
    EmptyList,
    Void,
    If(Box<If>),
    Let(Box<Let>),
    Define(Box<Define>),
    Lambda(Box<Lambda>),
    Assignment(Box<Assignment>),
    Begin(Begin),
    Cond(Cond),
    And(And),
    Or(Or),
    Quoted(Box<Expr>),
}

impl Expr {
    pub fn into_list(self) -> Result<Expr, EvalErr> {
        Ok(Expr::List(vec![self]))
    }

    fn try_valid_single(self) -> Result<Expr, EvalErr> {
        match self {
            f @ Expr::Atom(Token::Else) => Err(EvalErr::UnexpectedToken(f.printable())),
            t => Ok(t),
        }
    }
}

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    parsed_exprs: Vec<Expr>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
            parsed_exprs: vec![],
        }
    }

    pub fn parse(mut self) -> Result<Vec<Expr>, EvalErr> {
        while self.tokens.peek().is_some() {
            let expr = self.parse_from_token()?.try_valid_single()?;
            self.parsed_exprs.push(expr)
        }
        Ok(self.parsed_exprs)
    }

    fn parse_from_token(&mut self) -> Result<Expr, EvalErr> {
        match self.next_or_err(EvalErr::UnexpectedEnd)? {
            Token::LParen => match self.peek_or_err(EvalErr::UnexpectedEnd)? {
                Token::If => {
                    self.tokens.next();
                    self.parse_if()?.into_list()
                }
                Token::Lambda => {
                    self.tokens.next();
                    self.parse_lambda()?.into_list()
                }
                Token::Define => {
                    self.tokens.next();
                    self.parse_define()?.into_list()
                }
                Token::And => {
                    self.tokens.next();
                    self.parse_and()?.into_list()
                }
                Token::Or => {
                    self.tokens.next();
                    self.parse_or()?.into_list()
                }
                Token::Cond => {
                    self.tokens.next();
                    self.parse_cond()?.into_list()
                }
                Token::Assignment => {
                    self.tokens.next();
                    self.parse_assignment()?.into_list()
                }
                Token::Begin => {
                    self.tokens.next();
                    self.parse_begin()?.into_list()
                }
                Token::Let => {
                    self.tokens.next();
                    self.parse_let()?.into_list()
                }
                Token::QuoteProc => {
                    self.tokens.next();
                    let quoted = self.parse_quote();
                    self.next_or_err(EvalErr::UnexpectedEnd)?; // consume remaining paren
                    quoted
                }
                _ => self.parse_list(),
            },
            Token::QuoteTick => self.parse_quote(),
            x @ Token::Number(_)
            | x @ Token::Str(_)
            | x @ Token::Boolean(_)
            | x @ Token::Symbol(_)
            | x @ Token::Else => Ok(Expr::Atom(x)),
            t => Err(EvalErr::UnexpectedToken(t.printable())),
        }
    }

    fn parse_inner_list(&mut self) -> Result<Vec<Expr>, EvalErr> {
        let mut parsed_exprs: Vec<Expr> = vec![];
        while let Some(t) = self.tokens.peek() {
            match t {
                Token::RParen => {
                    self.tokens.next();
                    return Ok(parsed_exprs);
                }
                _ => parsed_exprs.push(self.parse_from_token()?),
            }
        }
        Err(EvalErr::UnexpectedEnd)
    }

    fn parse_list(&mut self) -> Result<Expr, EvalErr> {
        let list = self.parse_inner_list()?;
        match !list.is_empty() {
            true => Ok(list.to_expr()),
            false => Ok(Expr::EmptyList),
        }
    }

    fn parse_if(&mut self) -> Result<Expr, EvalErr> {
        let (predicate, consequence, alternative) =
            self.parse_inner_list()?.into_iter().get_three_or_else(|| {
                EvalErr::InvalidArgs(
                    "'if' expression. expected condition, predicate, and consequence",
                )
            })?;
        Ok(If::new(predicate, consequence, alternative).to_expr())
    }

    fn parse_cond(&mut self) -> Result<Expr, EvalErr> {
        let clauses = self.parse_inner_list()?;
        match !clauses.is_empty() {
            true => Ok(Cond::new(clauses).to_expr()),
            false => Err(EvalErr::InvalidArgs("'cond' expression. expected clauses.")),
        }
    }

    fn parse_lambda(&mut self) -> Result<Expr, EvalErr> {
        let (params, body) = self
            .parse_inner_list()?
            .into_iter()
            .get_one_and_rest_or_else(|| {
                EvalErr::InvalidArgs("'lambda' expression. expected parameters and body")
            })?;
        Ok(Lambda::new(params, body.collect()).to_expr())
    }

    fn parse_define(&mut self) -> Result<Expr, EvalErr> {
        let (identifier, body) = self
            .parse_inner_list()?
            .into_iter()
            .get_one_and_rest_or_else(|| {
                EvalErr::InvalidArgs("'define' expression. expected identifier and value")
            })?;
        Ok(Define::new(identifier, body.collect()).to_expr())
    }

    fn parse_assignment(&mut self) -> Result<Expr, EvalErr> {
        let (first, second) = self.parse_inner_list()?.into_iter().get_two_or_else(|| {
            EvalErr::InvalidArgs("'set!' expression. expected identifier and value")
        })?;
        Ok(Assignment::new(first, second).to_expr())
    }

    fn parse_let(&mut self) -> Result<Expr, EvalErr> {
        let (bindings, body) = self
            .parse_inner_list()?
            .into_iter()
            .get_one_and_rest_or_else(|| {
                EvalErr::InvalidArgs("'let' expression. expected bindings and body")
            })?;
        Ok(Let::new(bindings, body.collect()).to_expr())
    }

    fn parse_begin(&mut self) -> Result<Expr, EvalErr> {
        Ok(Begin::new(self.parse_inner_list()?).to_expr())
    }

    fn parse_and(&mut self) -> Result<Expr, EvalErr> {
        Ok(And::new(self.parse_inner_list()?).to_expr())
    }

    fn parse_or(&mut self) -> Result<Expr, EvalErr> {
        Ok(Or::new(self.parse_inner_list()?).to_expr())
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
            | t @ Token::Let
            | t @ Token::If
            | t @ Token::And
            | t @ Token::Cond
            | t @ Token::QuoteTick
            | t @ Token::QuoteProc
            | t @ Token::Else
            | t @ Token::Begin
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
                Token::RParen => Ok(Expr::EmptyList),
                _ => {
                    let current = self.parse_quote()?;
                    let next = self.parse_inner_quote()?;
                    Ok(Pair::new(current, next).to_expr())
                }
            },
            None => Err(EvalErr::UnexpectedEnd),
        }
    }

    fn next_or_err(&mut self, err: EvalErr) -> Result<Token, EvalErr> {
        self.tokens.next().map_or_else(|| Err(err), Ok)
    }

    fn peek_or_err(&mut self, err: EvalErr) -> Result<&Token, EvalErr> {
        self.tokens.peek().map_or_else(|| Err(err), Ok)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::TokenStream;

    #[test]
    fn valid_parse() {
        let scm = "1 (+ 1 (+ 1 2))";
        let res: Vec<Expr> = vec![
            1.0.to_expr(),
            vec![
                "+".to_string().to_expr(),
                1.0.to_expr(),
                vec!["+".to_string().to_expr(), 1.0.to_expr(), 2.0.to_expr()].to_expr(),
            ]
            .to_expr(),
        ];
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        let exprs = Parser::new(tokens).parse().unwrap();
        assert_eq!(res, exprs);
    }

    #[test]
    fn quoted() {
        let scm = "'(+ 1)";
        let res: Vec<Expr> = vec![Expr::Quoted(Box::new(
            Pair::new(
                Expr::Quoted(Box::new("+".to_string().to_expr())),
                Pair::new(Expr::Quoted(Box::new(1.0.to_expr())), Expr::EmptyList).to_expr(),
            )
            .to_expr(),
        ))];
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        let exprs = Parser::new(tokens).parse().unwrap();
        assert_eq!(res, exprs);
    }

    #[test]
    fn quoted_fn() {
        let scm = "(quote (+ 1))";
        let res: Vec<Expr> = vec![Expr::Quoted(Box::new(
            Pair::new(
                Expr::Quoted(Box::new(Expr::Atom(Token::Symbol("+".to_string())))),
                Pair::new(
                    Expr::Quoted(Box::new(Expr::Atom(Token::Number(1.0)))),
                    Expr::EmptyList,
                )
                .to_expr(),
            )
            .to_expr(),
        ))];
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        let exprs = Parser::new(tokens).parse().unwrap();
        assert_eq!(res, exprs);
    }

    #[test]
    #[should_panic]
    fn extra_paren() {
        let scm = "(+ 1 2) (1))";
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        Parser::new(tokens).parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn opening_rparen() {
        let scm = ")(yo)";
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        Parser::new(tokens).parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn no_close() {
        let scm = "(+ 1 (1)";
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        Parser::new(tokens).parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_else() {
        let scm = "else";
        let tokens = TokenStream::new(scm).collect_tokens().unwrap();
        Parser::new(tokens).parse().unwrap();
    }
}
