#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Char(char),
    Escape(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassElement {
    Range(char, char),
    Literal(Literal),
}
impl ClassElement {
    pub fn size(&self) -> usize {
        match *self {
            ClassElement::Range(a, b) => (b as usize) - (a as usize) + 1,
            ClassElement::Literal(_) => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Literal(Literal),
    Class(Vec<ClassElement>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Factor {
    Token(Token),
    Group(Box<Expression>),
    FixedGroup(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Factor(Factor),
    WithSuffix(Factor, TermSuffix),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermSuffix {
    Question,
    Asterisk,
    Plus,
    Range(usize, usize),
    OpenRange(usize),
    Repeat(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terms {
    Concat(Vec<Term>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Union(Vec<Terms>),
}
