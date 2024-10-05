#[allow(dead_code)]
pub mod ast {
    pub fn print() {
        let expr = Expression::Equality(Box::new(Equality::Comparison(Box::new(
            Comparison::Term(Box::new(Term::Binary(
                Box::new(Factor::Unary(Box::new(Unary::Primary(Box::new(
                    Primary::Number(1.0),
                ))))),
                TermOp::Add,
                Box::new(Factor::Unary(Box::new(Unary::Primary(Box::new(
                    Primary::Number(1.0),
                ))))),
            ))),
        ))));
        println!("{:?}", expr);
    }
    #[derive(Debug)]
    pub enum Expression {
        Equality(Box<Equality>),
    }

    #[derive(Debug)]
    pub enum Equality {
        Comparison(Box<Comparison>),
        Binary(Box<Comparison>, EqualityOp, Box<Comparison>),
    }
    #[derive(Debug)]

    pub enum EqualityOp {
        NotEqual,
        Equal,
    }
    #[derive(Debug)]

    pub enum Comparison {
        Term(Box<Term>),
        Binary(Box<Term>, ComparisonOp, Box<Term>),
    }
    #[derive(Debug)]

    pub enum ComparisonOp {
        Greater,
        GreaterEqual,
        Less,
        LessEqual,
    }
    #[derive(Debug)]

    pub enum Term {
        Factor(Box<Factor>),
        Binary(Box<Factor>, TermOp, Box<Factor>),
    }
    #[derive(Debug)]

    pub enum TermOp {
        Add,
        Subtract,
    }
    #[derive(Debug)]

    pub enum Factor {
        Unary(Box<Unary>),
        Binary(Box<Unary>, FactorOp, Box<Unary>),
    }
    #[derive(Debug)]

    pub enum FactorOp {
        Divide,
        Multiply,
    }
    #[derive(Debug)]

    pub enum Unary {
        Primary(Box<Primary>),
        UnaryOp(UnaryOp, Box<Unary>),
    }
    #[derive(Debug)]

    pub enum UnaryOp {
        Not,
        Negate,
    }
    #[derive(Debug)]

    pub enum Primary {
        Number(f64),
        String(String),
        True,
        False,
        Nil,
        Grouping(Box<Expression>),
    }
}