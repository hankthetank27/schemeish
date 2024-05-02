use crate::{
    lexer::Token,
    parser::Expr,
    primitives::pair::{MaybeList, Pair},
    procedure::Proc,
    special_form::MutCell,
};

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
            Token::Else => "else".into(),
            Token::Or => "or".into(),
            Token::Cond => "cond".into(),
            Token::QuoteTick => "'".into(),
            Token::QuoteProc => "quote".into(),
            Token::Symbol(s) => s.into(),
            Token::Number(n) => n.to_string(),
            Token::Boolean(b) => match b {
                true => "#t".into(),
                false => "#f".into(),
            },
            Token::MutatePair(p) => match p {
                MutCell::Car => "set-car!".into(),
                MutCell::Cdr => "set-cdr!".into(),
            },
            Token::Str(s) => format!(r##""{s}""##),
        }
    }
}

impl Printable for Proc {
    fn printable(&self) -> String {
        match self {
            Proc::Primitive(p) => format!("#<primitive-{:?}>", p.inner()),
            Proc::Compound(p) => format!("#<closure-(#f{})>", p.to_owned().params().printable()),
        }
    }
}

impl Printable for Expr {
    fn printable(&self) -> String {
        match self {
            Expr::EmptyList => "'()".to_string(),
            Expr::Atom(a) => a.printable(),
            Expr::Proc(p) => p.printable(),
            Expr::List(l) => l.printable(),
            Expr::Dotted(p) => p.printable(),
            Expr::Quoted(q) => (*q).printable(),
            x => format!("{:?}", x),
        }
    }
}

impl Printable for Vec<Expr> {
    fn printable(&self) -> String {
        let ls = self
            .iter()
            .map(|e| e.printable())
            .reduce(|curr, next| format!("{} {}", curr, next))
            .unwrap();
        format!("'({})", ls)
    }
}

impl Printable for Vec<String> {
    fn printable(&self) -> String {
        let ls = self
            .iter()
            .map(|e| e.to_string())
            .reduce(|curr, next| format!("{} {}", curr, next))
            .unwrap();
        format!("'({})", ls)
    }
}

impl Printable for Pair {
    fn printable(&self) -> String {
        match self.try_list() {
            MaybeList::List(ls) => match ls {
                Some(ls) => format!(
                    "'({})",
                    ls.iter()
                        .map(|e| e.printable())
                        .reduce(|curr, next| format!("{} {}", curr, next))
                        .unwrap()
                ),
                None => Expr::EmptyList.printable(),
            },
            MaybeList::Pair(p) => format!("({} . {})", p.car.printable(), p.cdr.printable()),
        }
    }
}
