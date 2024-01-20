use std::iter::Peekable;

use crate::lexer::Token;

#[derive(Debug, PartialEq)]
pub enum Expr {
    List(Vec<Expr>),
    Atom(Token),
}

pub fn parse(tokens: Vec<Token>) -> Vec<Expr> {
    let mut tokens = tokens.into_iter().peekable();
    let mut exprs: Vec<Expr> = vec![];

    while let Some(_) = tokens.peek() {
        exprs.push(read_from_tokens(&mut tokens))
    }

    exprs
}

fn read_from_tokens<T: Iterator<Item = Token>>(tokens: &mut Peekable<T>) -> Expr {
    match tokens.peek() {
        Some(Token::RParen) => panic!("Unexpected ')'"),
        Some(Token::LParen) => {
            let mut exprs: Vec<Expr> = vec![];
            tokens.next();
            while tokens.peek() != Some(&Token::RParen) {
                exprs.push(read_from_tokens(tokens))
            }
            tokens.next();
            Expr::List(exprs)
        }
        Some(_) => Expr::Atom(tokens.next().unwrap()),
        None => panic!("Unexpected EOF"),
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
        assert_eq!(res, parse(tokenize(scm)));
    }

    #[test]
    #[should_panic]
    fn extra_paren() {
        let scm = "(+ 1 2) (1))";
        parse(tokenize(scm));
    }

    #[test]
    #[should_panic]
    fn opening_rparen() {
        let scm = ")(yo)";
        parse(tokenize(scm));
    }

    #[test]
    #[should_panic]
    fn no_close() {
        let scm = "(+ 1 (1)";
        parse(tokenize(scm));
    }
}
