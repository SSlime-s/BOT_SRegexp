use nom::{multi::many1, sequence::delimited, IResult};

use crate::model::{ClassElement, Expression, Factor, Literal, Term, TermSuffix, Terms, Token};

fn literal(s: &str) -> IResult<&str, Literal> {
    let (s, first) = nom::character::complete::anychar(s)?;
    if first == '\\' {
        let (s, second) = nom::character::complete::anychar(s)?;
        return Ok((s, Literal::Escape(second)));
    }

    // reserved characters
    if matches!(
        first,
        '[' | ']' | '(' | ')' | '<' | '>' | '{' | '}' | '?' | '*' | '+' | '|'
    ) {
        return Err(nom::Err::Error(nom::error::Error::new(
            s,
            nom::error::ErrorKind::Tag,
        )));
    }

    Ok((s, Literal::Char(first)))
}

fn class_element(s: &str) -> IResult<&str, ClassElement> {
    let (s, first) = literal(s)?;
    if first == Literal::Char('-') || matches!(first, Literal::Escape(_)) {
        return Ok((s, ClassElement::Literal(first)));
    }
    let first_char = match first {
        Literal::Char(c) => c,
        Literal::Escape(_) => unreachable!(),
    };

    let res = nom::character::complete::char::<_, nom::error::Error<&str>>('-')(s);
    match res {
        Ok((s2, _)) => {
            let res = literal(s2);
            match res {
                Ok((s, Literal::Char(second))) => {
                    if first_char > second {
                        return Err(nom::Err::Error(nom::error::Error::new(
                            s,
                            nom::error::ErrorKind::Tag,
                        )));
                    }
                    Ok((s, ClassElement::Range(first_char, second)))
                }
                Ok((_, Literal::Escape(_))) => Ok((s, ClassElement::Literal(first))),
                Err(_) => Ok((s, ClassElement::Literal(first))),
            }
        }
        Err(_) => Ok((s, ClassElement::Literal(first))),
    }
}

fn token(s: &str) -> IResult<&str, Token> {
    match delimited(
        nom::character::complete::char('['),
        many1(class_element),
        nom::character::complete::char(']'),
    )(s)
    {
        Ok((s, elements)) => Ok((s, Token::Class(elements))),
        Err(_) => {
            let (s, literal) = literal(s)?;
            Ok((s, Token::Literal(literal)))
        }
    }
}

fn factor(s: &str) -> IResult<&str, Factor> {
    match delimited(
        nom::character::complete::char('('),
        expression,
        nom::character::complete::char(')'),
    )(s)
    {
        Ok((s, expression)) => Ok((s, Factor::Group(Box::new(expression)))),
        Err(_) => match delimited(
            nom::character::complete::char('<'),
            expression,
            nom::character::complete::char('>'),
        )(s)
        {
            Ok((s, expression)) => Ok((s, Factor::FixedGroup(Box::new(expression)))),
            Err(_) => {
                let (s, token) = token(s)?;
                Ok((s, Factor::Token(token)))
            }
        },
    }
}

fn term(s: &str) -> IResult<&str, Term> {
    let (s, first) = factor(s)?;
    let (s, suffix) = nom::combinator::opt(term_suffix)(s)?;

    match suffix {
        Some(suffix) => Ok((s, Term::WithSuffix(first, suffix))),
        None => Ok((s, Term::Factor(first))),
    }
}

fn term_suffix(s: &str) -> IResult<&str, TermSuffix> {
    let (s2, first) = nom::branch::alt((
        nom::character::complete::char('?'),
        nom::character::complete::char('*'),
        nom::character::complete::char('+'),
        nom::character::complete::char('{'),
    ))(s)?;

    match first {
        '?' => Ok((s2, TermSuffix::Question)),
        '*' => Ok((s2, TermSuffix::Asterisk)),
        '+' => Ok((s2, TermSuffix::Plus)),
        '{' => {
            let (s, t) = delimited(
                nom::character::complete::char('{'),
                nom::branch::permutation((
                    nom::character::complete::digit1,
                    nom::combinator::opt(nom::branch::permutation((
                        nom::character::complete::char(','),
                        nom::combinator::opt(nom::character::complete::digit1),
                    ))),
                )),
                nom::character::complete::char('}'),
            )(s2)?;

            let digit_error =
                |_| nom::Err::Error(nom::error::Error::new(s, nom::error::ErrorKind::Digit));
            match t {
                (min, None) => Ok((s, TermSuffix::Repeat(min.parse().map_err(digit_error)?))),
                (min, Some((_, None))) => {
                    Ok((s, TermSuffix::OpenRange(min.parse().map_err(digit_error)?)))
                }
                (min, Some((_, Some(max)))) => Ok((
                    s,
                    TermSuffix::Range(
                        min.parse().map_err(digit_error)?,
                        max.parse().map_err(digit_error)?,
                    ),
                )),
            }
        }
        _ => unreachable!(),
    }
}

fn terms(s: &str) -> IResult<&str, Terms> {
    let (s, terms) = many1(term)(s)?;
    Ok((s, Terms::Concat(terms)))
}

pub fn expression(s: &str) -> IResult<&str, Expression> {
    let (s, contents) = nom::multi::separated_list1(nom::character::complete::char('|'), terms)(s)?;
    Ok((s, Expression::Union(contents)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_expr() -> Expression {
        Expression::Union(vec![Terms::Concat(vec![Term::Factor(Factor::Token(
            Token::Literal(Literal::Char('a')),
        ))])])
    }

    fn a_or_b_expr() -> Expression {
        Expression::Union(vec![
            Terms::Concat(vec![Term::Factor(Factor::Token(Token::Literal(
                Literal::Char('a'),
            )))]),
            Terms::Concat(vec![Term::Factor(Factor::Token(Token::Literal(
                Literal::Char('b'),
            )))]),
        ])
    }

    fn a_or_b_or_c_expr() -> Expression {
        Expression::Union(vec![
            Terms::Concat(vec![Term::Factor(Factor::Token(Token::Literal(
                Literal::Char('a'),
            )))]),
            Terms::Concat(vec![Term::Factor(Factor::Token(Token::Literal(
                Literal::Char('b'),
            )))]),
            Terms::Concat(vec![Term::Factor(Factor::Token(Token::Literal(
                Literal::Char('c'),
            )))]),
        ])
    }

    #[test]
    fn test_literal() {
        assert_eq!(literal("a"), Ok(("", Literal::Char('a'))));
        assert_eq!(literal("ab"), Ok(("b", Literal::Char('a'))));
        assert_eq!(literal(r"\a"), Ok(("", Literal::Escape('a'))));
        assert_eq!(literal(r"\ab"), Ok(("b", Literal::Escape('a'))));
    }

    #[test]
    fn test_class_element() {
        assert_eq!(
            class_element("a"),
            Ok(("", ClassElement::Literal(Literal::Char('a'))))
        );
        assert_eq!(
            class_element(r"\a"),
            Ok(("", ClassElement::Literal(Literal::Escape('a'))))
        );
        assert_eq!(
            class_element(r"a-z"),
            Ok(("", ClassElement::Range('a', 'z')))
        );
        assert_eq!(
            class_element(r"\a-z"),
            Ok(("-z", ClassElement::Literal(Literal::Escape('a'))))
        );
        assert_eq!(
            class_element(r"a-\z"),
            Ok((r"-\z", ClassElement::Literal(Literal::Char('a'))))
        );
        assert_eq!(
            class_element(r"-a-z"),
            Ok(("a-z", ClassElement::Literal(Literal::Char('-'))))
        );
        assert_eq!(
            class_element(r"a-z-"),
            Ok(("-", ClassElement::Range('a', 'z')))
        );
        assert_eq!(
            class_element(r"a-"),
            Ok(("-", ClassElement::Literal(Literal::Char('a'))))
        );
    }

    #[test]
    fn test_token_class() {
        assert_eq!(
            token("[a]"),
            Ok((
                "",
                Token::Class(vec![ClassElement::Literal(Literal::Char('a'))])
            ))
        );
        assert_eq!(
            token("[ab]"),
            Ok((
                "",
                Token::Class(vec![
                    ClassElement::Literal(Literal::Char('a')),
                    ClassElement::Literal(Literal::Char('b'))
                ])
            ))
        );
        assert_eq!(
            token("[a-z]"),
            Ok(("", Token::Class(vec![ClassElement::Range('a', 'z')])))
        );
        assert_eq!(
            token("[a-z0-9]"),
            Ok((
                "",
                Token::Class(vec![
                    ClassElement::Range('a', 'z'),
                    ClassElement::Range('0', '9')
                ])
            ))
        );
        assert_eq!(
            token("[a-z0-9-]"),
            Ok((
                "",
                Token::Class(vec![
                    ClassElement::Range('a', 'z'),
                    ClassElement::Range('0', '9'),
                    ClassElement::Literal(Literal::Char('-'))
                ])
            ))
        );
        assert_eq!(
            token(r"[a-z0-9\]]"),
            Ok((
                "",
                Token::Class(vec![
                    ClassElement::Range('a', 'z'),
                    ClassElement::Range('0', '9'),
                    ClassElement::Literal(Literal::Escape(']'))
                ])
            ))
        );
        assert_eq!(
            token(r"[\]]"),
            Ok((
                "",
                Token::Class(vec![ClassElement::Literal(Literal::Escape(']'))])
            ))
        );
    }

    #[test]
    fn test_token_literal() {
        assert_eq!(token("a"), Ok(("", Token::Literal(Literal::Char('a')))));
        assert_eq!(token(r"\a"), Ok(("", Token::Literal(Literal::Escape('a')))));
        assert_eq!(
            token(r"\ab"),
            Ok(("b", Token::Literal(Literal::Escape('a'))))
        );
    }

    #[test]
    fn test_expression() {
        assert_eq!(expression("a"), Ok(("", a_expr())));
        assert_eq!(expression("a|b"), Ok(("", a_or_b_expr())));
        assert_eq!(expression("a|b|c"), Ok(("", a_or_b_or_c_expr())));
    }

    #[test]
    fn test_factor_group() {
        assert_eq!(factor("(a)"), Ok(("", Factor::Group(Box::new(a_expr())))));
        assert_eq!(
            factor("(a|b)"),
            Ok(("", Factor::Group(Box::new(a_or_b_expr()))))
        );
        assert_eq!(
            factor("(a|b|c)"),
            Ok(("", Factor::Group(Box::new(a_or_b_or_c_expr()))))
        );

        assert_eq!(
            factor("(a|b)c"),
            Ok(("c", Factor::Group(Box::new(a_or_b_expr()))))
        );
    }

    #[test]
    fn test_factor_fixed_group() {
        assert_eq!(
            factor("<a>"),
            Ok(("", Factor::FixedGroup(Box::new(a_expr()))))
        );
        assert_eq!(
            factor("<a|b>"),
            Ok(("", Factor::FixedGroup(Box::new(a_or_b_expr()))))
        );
        assert_eq!(
            factor("<a|b|c>"),
            Ok(("", Factor::FixedGroup(Box::new(a_or_b_or_c_expr()))))
        );

        assert_eq!(
            factor("<a|b>c"),
            Ok(("c", Factor::FixedGroup(Box::new(a_or_b_expr()))))
        );
    }
}
