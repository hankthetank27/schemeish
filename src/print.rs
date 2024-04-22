use crate::{lexer::Token, parser::Expr, procedure::Proc};

pub trait Print {
    fn print(self) -> String;
}

impl Print for Token {
    fn print(self) -> String {
        match self {
            Token::LParen => "(".into(),
            Token::RParen => ")".into(),
            Token::If => "if".into(),
            Token::Define => "define".into(),
            Token::Lambda => "lambda".into(),
            Token::Assignment => "set!".into(),
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

impl Print for Proc {
    fn print(self) -> String {
        match self {
            Proc::Primitive(p) => format!("#<primitive-{:?}>", p.inner()),
            Proc::Compound(p) => format!("#<closure-(#f{:?})>", p.params()),
        }
    }
}

impl Print for Expr {
    fn print(self) -> String {
        match self {
            Expr::Atom(a) => a.print(),
            Expr::Proc(p) => p.print(),
            x => format!("{:?}", x),
        }
    }
}
// impl<T> Print for Vec<T> {
//     fn print(&self) -> String {}
// }
