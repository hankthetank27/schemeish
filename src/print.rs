use crate::{lexer::Token, parser::Expr, procedure::Proc};

pub trait Print<T> {
    fn print(&self);
}

impl<T: Printable> Print<T> for T {
    fn print(&self) {
        println!("{}", self.printable())
    }
}

pub trait Printable {
    fn printable(&self) -> String;
}

impl Printable for Token {
    fn printable(&self) -> String {
        match self {
            Token::LParen => "(".into(),
            Token::RParen => ")".into(),
            Token::If => "if".into(),
            Token::Define => "define".into(),
            Token::Lambda => "lambda".into(),
            Token::Assignment => "set!".into(),
            Token::And => "and".into(),
            Token::Or => "or".into(),
            Token::Quote => "'".into(),
            Token::Symbol(s) => s.into(),
            Token::Number(n) => n.to_string(),
            Token::Boolean(b) => match b {
                true => "#t".into(),
                false => "#f".into(),
            },
            Token::Str(s) => format!(r##""{s}""##),
        }
    }
}

impl Printable for Proc {
    fn printable(&self) -> String {
        match self {
            Proc::Primitive(p) => format!("#<primitive-{:?}>", p.inner()),
            Proc::Compound(p) => format!("#<closure-(#f{:?})>", p.to_owned().params()),
        }
    }
}

impl Printable for Expr {
    fn printable(&self) -> String {
        match self {
            Expr::Atom(a) => a.printable(),
            Expr::Proc(p) => p.printable(),
            x => format!("{:?}", x),
        }
    }
}
// impl<T> Print for Vec<T> {
//     fn print(&self) -> String {}
// }
