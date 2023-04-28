use crate::model::{ClassElement, Expression, Factor, Literal, Term, TermSuffix, Terms, Token};
use anyhow::Result;
use rand::Rng;

pub trait Generate {
    fn generate(&self, rng: &mut impl Rng) -> Result<String>;
}

impl Generate for Literal {
    fn generate(&self, _rng: &mut impl Rng) -> Result<String> {
        match self {
            Literal::Char(c) => Ok(c.to_string()),
            Literal::Escape('d') => Ok(rand::thread_rng().gen_range(0..=9).to_string()),
            Literal::Escape('w') => {
                let mut rng = rand::thread_rng();
                let c = match rng.gen_range(0..=61) {
                    x @ 0..=25 => b'a' + x,
                    x @ 26..=51 => b'A' + x - 26,
                    x @ 52..=61 => b'0' + x - 52,
                    _ => b'_',
                };
                Ok(std::char::from_u32(c as u32)
                    .ok_or_else(|| anyhow::anyhow!("Invalid range"))?
                    .to_string())
            }
            Literal::Escape('a'..='z') => anyhow::bail!("Invalid escape"),
            Literal::Escape('A'..='Z') => anyhow::bail!("Invalid escape"),
            Literal::Escape('0'..='9') => anyhow::bail!("Invalid escape"),
            Literal::Escape(c) => Ok(c.to_string()),
        }
    }
}

impl Generate for ClassElement {
    fn generate(&self, rng: &mut impl Rng) -> Result<String> {
        match self {
            ClassElement::Range(a, b) => {
                let a = *a as usize;
                let b = *b as usize;

                anyhow::ensure!(a <= b, "Invalid range");

                let c = rng.gen_range(a..=b);
                Ok(std::char::from_u32(c as u32)
                    .ok_or_else(|| anyhow::anyhow!("Invalid range"))?
                    .to_string())
            }
            ClassElement::Literal(l) => l.generate(rng),
        }
    }
}
impl ClassElement {
    pub fn size(&self) -> usize {
        match *self {
            ClassElement::Range(a, b) => (b as usize) - (a as usize) + 1,
            ClassElement::Literal(Literal::Escape('d')) => 10,
            ClassElement::Literal(Literal::Escape('w')) => 26 + 26 + 10 + 1,
            ClassElement::Literal(_) => 1,
        }
    }
}

impl Generate for Token {
    fn generate(&self, rng: &mut impl Rng) -> Result<String> {
        match self {
            Token::Literal(l) => l.generate(rng),
            Token::Class(c) => {
                let sum = c.iter().map(|e| e.size()).sum::<usize>();
                let mut r = rng.gen_range(0..sum);
                for e in c {
                    let s = e.size();
                    if r < s {
                        return e.generate(rng);
                    }
                    r -= s;
                }
                anyhow::bail!("Invalid class")
            }
        }
    }
}

impl Generate for Factor {
    fn generate(&self, rng: &mut impl Rng) -> Result<String> {
        match self {
            Factor::Token(t) => t.generate(rng),
            Factor::Group(e) => e.generate(rng),
            Factor::FixedGroup(e) => e.generate(rng),
        }
    }
}

impl Generate for Term {
    fn generate(&self, rng: &mut impl Rng) -> Result<String> {
        match self {
            Term::Factor(f) => f.generate(rng),
            Term::WithSuffix(f, s) => {
                let n = match s {
                    TermSuffix::Question => rng.gen_range(0..=1),
                    TermSuffix::Asterisk => {
                        let mut n = 0;
                        while rng.gen_bool(0.5) {
                            n += 1;
                        }
                        n
                    }
                    TermSuffix::Plus => {
                        let mut n = 1;
                        while rng.gen_bool(0.5) {
                            n += 1;
                        }
                        n
                    }
                    TermSuffix::Range(a, b) => rng.gen_range(*a..=*b),
                    TermSuffix::OpenRange(a) => {
                        let mut n = *a;
                        while rng.gen_bool(0.5) {
                            n += 1;
                        }
                        n
                    }
                    TermSuffix::Repeat(a) => *a,
                };
                match f {
                    Factor::FixedGroup(_) => {
                        let base = f.generate(rng)?;
                        Ok((0..n).map(|_| base.clone()).collect::<Vec<_>>().join(""))
                    }
                    _ => Ok((0..n)
                        .map(|_| f.generate(rng))
                        .collect::<Result<Vec<_>>>()?
                        .join("")),
                }
            }
        }
    }
}

impl Generate for Terms {
    fn generate(&self, rng: &mut impl Rng) -> Result<String> {
        match self {
            Terms::Concat(t) => t
                .iter()
                .map(|t| t.generate(rng))
                .collect::<Result<Vec<_>>>()
                .map(|v| v.join("")),
        }
    }
}

impl Generate for Expression {
    fn generate(&self, rng: &mut impl Rng) -> Result<String> {
        match self {
            Expression::Union(t) => {
                let i = rng.gen_range(0..t.len());
                t[i].generate(rng)
            }
        }
    }
}
