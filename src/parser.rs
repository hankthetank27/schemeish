use std::iter::Peekable;

use crate::error::ParseErr;
use crate::lexer::Token;
use crate::primitives::pair::Pair;
use crate::procedure::Proc;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    List(Vec<Expr>),
    Atom(Token),
    Proc(Proc),
    Dotted(Pair),
    EmptyList,
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Expr>, ParseErr> {
    let mut tokens = tokens.into_iter().peekable();
    let mut exprs: Vec<Expr> = vec![];

    while tokens.peek().is_some() {
        exprs.push(read_from_tokens(&mut tokens)?)
    }

    Ok(exprs)
}

fn read_from_tokens<T>(tokens: &mut Peekable<T>) -> Result<Expr, ParseErr>
where
    T: Iterator<Item = Token>,
{
    match tokens.peek() {
        Some(Token::RParen) => Err(ParseErr::UnexpectedToken("unexpected )".to_string())),
        Some(Token::LParen) => {
            let mut exprs: Vec<Expr> = vec![];
            tokens.next();
            while tokens.peek() != Some(&Token::RParen) {
                exprs.push(read_from_tokens(tokens)?)
            }
            tokens.next();
            Ok(Expr::List(exprs))
        }
        Some(_) => Ok(Expr::Atom(tokens.next().unwrap())),
        None => Err(ParseErr::UnexpectedEnd),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::tokenize;

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
        assert_eq!(res, parse(tokenize(scm).unwrap()).unwrap());
    }

    #[test]
    #[should_panic]
    fn extra_paren() {
        let scm = "(+ 1 2) (1))";
        parse(tokenize(scm).unwrap()).unwrap();
    }

    #[test]
    #[should_panic]
    fn opening_rparen() {
        let scm = ")(yo)";
        parse(tokenize(scm).unwrap()).unwrap();
    }

    #[test]
    #[should_panic]
    fn no_close() {
        let scm = "(+ 1 (1)";
        parse(tokenize(scm).unwrap()).unwrap();
    }
}
