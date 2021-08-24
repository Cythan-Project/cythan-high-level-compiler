// pest. The Elegant Parser
// Copyright (c) 2018 Dragoș Tiselice
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::char;
use std::iter::Peekable;
use std::rc::Rc;

use pest::error::{Error, ErrorVariant, LocatedError};
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::{Parser, Span};

use ast::{Expr, Rule as AstRule, RuleType};
use validator;

mod grammar {
    #![allow(unknown_lints, clippy::all)]

    include!("grammar.rs");
}

pub use self::grammar::*;

pub fn parse(rule: Rule, data: Rc<String>) -> Result<Pairs<Rule>, Error<Rule>> {
    PestParser::parse(rule, data)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParserRule {
    pub name: String,
    pub span: Span,
    pub ty: RuleType,
    pub node: ParserNode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParserNode {
    pub expr: ParserExpr,
    pub span: Span,
}

impl ParserNode {
    pub fn filter_map_top_down<F, T>(self, mut f: F) -> Vec<T>
    where
        F: FnMut(ParserNode) -> Option<T>,
    {
        pub fn filter_internal<F, T>(node: ParserNode, f: &mut F, result: &mut Vec<T>)
        where
            F: FnMut(ParserNode) -> Option<T>,
        {
            if let Some(value) = f(node.clone()) {
                result.push(value);
            }

            match node.expr {
                // TODO: Use box syntax when it gets stabilized.
                ParserExpr::PosPred(node) => {
                    filter_internal(*node, f, result);
                }
                ParserExpr::NegPred(node) => {
                    filter_internal(*node, f, result);
                }
                ParserExpr::Seq(lhs, rhs) => {
                    filter_internal(*lhs, f, result);
                    filter_internal(*rhs, f, result);
                }
                ParserExpr::Choice(lhs, rhs) => {
                    filter_internal(*lhs, f, result);
                    filter_internal(*rhs, f, result);
                }
                ParserExpr::Rep(node) => {
                    filter_internal(*node, f, result);
                }
                ParserExpr::RepOnce(node) => {
                    filter_internal(*node, f, result);
                }
                ParserExpr::RepExact(node, _) => {
                    filter_internal(*node, f, result);
                }
                ParserExpr::RepMin(node, _) => {
                    filter_internal(*node, f, result);
                }
                ParserExpr::RepMax(node, _) => {
                    filter_internal(*node, f, result);
                }
                ParserExpr::RepMinMax(node, ..) => {
                    filter_internal(*node, f, result);
                }
                ParserExpr::Opt(node) => {
                    filter_internal(*node, f, result);
                }
                ParserExpr::Push(node) => {
                    filter_internal(*node, f, result);
                }
                _ => (),
            }
        }

        let mut result = vec![];

        filter_internal(self, &mut f, &mut result);

        result
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParserExpr {
    Str(String),
    Insens(String),
    Range(String, String),
    Ident(String),
    PeekSlice(i32, Option<i32>),
    PosPred(Box<ParserNode>),
    NegPred(Box<ParserNode>),
    Seq(Box<ParserNode>, Box<ParserNode>),
    Choice(Box<ParserNode>, Box<ParserNode>),
    Opt(Box<ParserNode>),
    Rep(Box<ParserNode>),
    RepOnce(Box<ParserNode>),
    RepExact(Box<ParserNode>, u32),
    RepMin(Box<ParserNode>, u32),
    RepMax(Box<ParserNode>, u32),
    RepMinMax(Box<ParserNode>, u32, u32),
    Push(Box<ParserNode>),
}

fn convert_rule(rule: ParserRule) -> AstRule {
    let ParserRule { name, ty, node, .. } = rule;
    let expr = convert_node(node);
    AstRule { name, ty, expr }
}

fn convert_node(node: ParserNode) -> Expr {
    match node.expr {
        ParserExpr::Str(string) => Expr::Str(string),
        ParserExpr::Insens(string) => Expr::Insens(string),
        ParserExpr::Range(start, end) => Expr::Range(start, end),
        ParserExpr::Ident(ident) => Expr::Ident(ident),
        ParserExpr::PeekSlice(start, end) => Expr::PeekSlice(start, end),
        ParserExpr::PosPred(node) => Expr::PosPred(Box::new(convert_node(*node))),
        ParserExpr::NegPred(node) => Expr::NegPred(Box::new(convert_node(*node))),
        ParserExpr::Seq(node1, node2) => Expr::Seq(
            Box::new(convert_node(*node1)),
            Box::new(convert_node(*node2)),
        ),
        ParserExpr::Choice(node1, node2) => Expr::Choice(
            Box::new(convert_node(*node1)),
            Box::new(convert_node(*node2)),
        ),
        ParserExpr::Opt(node) => Expr::Opt(Box::new(convert_node(*node))),
        ParserExpr::Rep(node) => Expr::Rep(Box::new(convert_node(*node))),
        ParserExpr::RepOnce(node) => Expr::RepOnce(Box::new(convert_node(*node))),
        ParserExpr::RepExact(node, num) => Expr::RepExact(Box::new(convert_node(*node)), num),
        ParserExpr::RepMin(node, max) => Expr::RepMin(Box::new(convert_node(*node)), max),
        ParserExpr::RepMax(node, max) => Expr::RepMax(Box::new(convert_node(*node)), max),
        ParserExpr::RepMinMax(node, min, max) => {
            Expr::RepMinMax(Box::new(convert_node(*node)), min, max)
        }
        ParserExpr::Push(node) => Expr::Push(Box::new(convert_node(*node))),
    }
}

pub fn consume_rules(pairs: Pairs<Rule>) -> Result<Vec<AstRule>, Vec<Error<Rule>>> {
    let rules = consume_rules_with_spans(pairs)?;
    let errors = validator::validate_ast(&rules);
    if errors.is_empty() {
        Ok(rules.into_iter().map(convert_rule).collect())
    } else {
        Err(errors)
    }
}

fn consume_rules_with_spans(pairs: Pairs<Rule>) -> Result<Vec<ParserRule>, Vec<Error<Rule>>> {
    let climber = PrecClimber::new(vec![
        Operator::new(Rule::choice_operator, Assoc::Left),
        Operator::new(Rule::sequence_operator, Assoc::Left),
    ]);

    pairs
        .filter(|pair| pair.as_rule() == Rule::grammar_rule)
        .map(|pair| {
            let mut pairs = pair.into_inner().peekable();

            let span = pairs.next().unwrap().as_span();
            let name = span.as_str().to_owned();

            pairs.next().unwrap(); // assignment_operator

            let ty = if pairs.peek().unwrap().as_rule() != Rule::opening_brace {
                match pairs.next().unwrap().as_rule() {
                    Rule::silent_modifier => RuleType::Silent,
                    Rule::atomic_modifier => RuleType::Atomic,
                    Rule::compound_atomic_modifier => RuleType::CompoundAtomic,
                    Rule::non_atomic_modifier => RuleType::NonAtomic,
                    _ => unreachable!(),
                }
            } else {
                RuleType::Normal
            };

            pairs.next().unwrap(); // opening_brace

            let node = consume_expr(pairs.next().unwrap().into_inner().peekable(), &climber)?;

            Ok(ParserRule {
                name,
                span,
                ty,
                node,
            })
        })
        .collect()
}

fn consume_expr(
    pairs: Peekable<Pairs<Rule>>,
    climber: &PrecClimber<Rule>,
) -> Result<ParserNode, Vec<Error<Rule>>> {
    fn unaries(
        mut pairs: Peekable<Pairs<Rule>>,
        climber: &PrecClimber<Rule>,
    ) -> Result<ParserNode, Vec<Error<Rule>>> {
        let pair = pairs.next().unwrap();

        let node = match pair.as_rule() {
            Rule::opening_paren => {
                let node = unaries(pairs, climber)?;
                let end = node.span.end_pos();

                ParserNode {
                    expr: node.expr,
                    span: pair.as_span().start_pos().span(&end),
                }
            }
            Rule::positive_predicate_operator => {
                let node = unaries(pairs, climber)?;
                let end = node.span.end_pos();

                ParserNode {
                    expr: ParserExpr::PosPred(Box::new(node)),
                    span: pair.as_span().start_pos().span(&end),
                }
            }
            Rule::negative_predicate_operator => {
                let node = unaries(pairs, climber)?;
                let end = node.span.end_pos();

                ParserNode {
                    expr: ParserExpr::NegPred(Box::new(node)),
                    span: pair.as_span().start_pos().span(&end),
                }
            }
            other_rule => {
                let node = match other_rule {
                    Rule::expression => consume_expr(pair.into_inner().peekable(), climber)?,
                    Rule::_push => {
                        let start = pair.clone().as_span().start_pos();
                        let mut pairs = pair.into_inner();
                        pairs.next().unwrap(); // opening_paren
                        let pair = pairs.next().unwrap();

                        let node = consume_expr(pair.into_inner().peekable(), climber)?;
                        let end = node.span.end_pos();

                        ParserNode {
                            expr: ParserExpr::Push(Box::new(node)),
                            span: start.span(&end),
                        }
                    }
                    Rule::peek_slice => {
                        let mut pairs = pair.clone().into_inner();
                        pairs.next().unwrap(); // opening_brack
                        let pair_start = pairs.next().unwrap(); // .. or integer
                        let start: i32 = match pair_start.as_rule() {
                            Rule::range_operator => 0,
                            Rule::integer => {
                                pairs.next().unwrap(); // ..
                                pair_start.as_str().parse().unwrap()
                            }
                            _ => unreachable!(),
                        };
                        let pair_end = pairs.next().unwrap(); // integer or }
                        let end: Option<i32> = match pair_end.as_rule() {
                            Rule::closing_brack => None,
                            Rule::integer => {
                                pairs.next().unwrap(); // }
                                Some(pair_end.as_str().parse().unwrap())
                            }
                            _ => unreachable!(),
                        };
                        ParserNode {
                            expr: ParserExpr::PeekSlice(start, end),
                            span: pair.as_span(),
                        }
                    }
                    Rule::identifier => ParserNode {
                        expr: ParserExpr::Ident(pair.as_str().to_owned()),
                        span: pair.clone().as_span(),
                    },
                    Rule::string => {
                        let string = unescape(pair.as_str()).expect("incorrect string literal");
                        ParserNode {
                            expr: ParserExpr::Str(string[1..string.len() - 1].to_owned()),
                            span: pair.clone().as_span(),
                        }
                    }
                    Rule::insensitive_string => {
                        let string = unescape(pair.as_str()).expect("incorrect string literal");
                        ParserNode {
                            expr: ParserExpr::Insens(string[2..string.len() - 1].to_owned()),
                            span: pair.clone().as_span(),
                        }
                    }
                    Rule::range => {
                        let mut pairs = pair.into_inner();
                        let pair = pairs.next().unwrap();
                        let start = unescape(pair.as_str()).expect("incorrect char literal");
                        let start_pos = pair.clone().as_span().start_pos();
                        pairs.next();
                        let pair = pairs.next().unwrap();
                        let end = unescape(pair.as_str()).expect("incorrect char literal");
                        let end_pos = pair.clone().as_span().end_pos();

                        ParserNode {
                            expr: ParserExpr::Range(
                                start[1..start.len() - 1].to_owned(),
                                end[1..end.len() - 1].to_owned(),
                            ),
                            span: start_pos.span(&end_pos),
                        }
                    }
                    _ => unreachable!(),
                };

                pairs.fold(
                    Ok(node),
                    |node: Result<ParserNode, Vec<Error<Rule>>>, pair| {
                        let node = node?;

                        let node = match pair.as_rule() {
                            Rule::optional_operator => {
                                let start = node.span.start_pos();
                                ParserNode {
                                    expr: ParserExpr::Opt(Box::new(node)),
                                    span: start.span(&pair.as_span().end_pos()),
                                }
                            }
                            Rule::repeat_operator => {
                                let start = node.span.start_pos();
                                ParserNode {
                                    expr: ParserExpr::Rep(Box::new(node)),
                                    span: start.span(&pair.as_span().end_pos()),
                                }
                            }
                            Rule::repeat_once_operator => {
                                let start = node.span.start_pos();
                                ParserNode {
                                    expr: ParserExpr::RepOnce(Box::new(node)),
                                    span: start.span(&pair.as_span().end_pos()),
                                }
                            }
                            Rule::repeat_exact => {
                                let mut inner = pair.clone().into_inner();

                                inner.next().unwrap(); // opening_brace

                                let number = inner.next().unwrap();
                                let num = if let Ok(num) = number.as_str().parse::<u32>() {
                                    num
                                } else {
                                    return Err(vec![Error::new(
                                        ErrorVariant::CustomError {
                                            message: "number cannot overflow u32".to_owned(),
                                        },
                                        vec![LocatedError::new_from_span(number.as_span())],
                                    )]);
                                };

                                if num == 0 {
                                    let error: Error<Rule> = Error::new(
                                        ErrorVariant::CustomError {
                                            message: "cannot repeat 0 times".to_owned(),
                                        },
                                        vec![LocatedError::new_from_span(number.as_span())],
                                    );

                                    return Err(vec![error]);
                                }

                                let start = node.span.start_pos();
                                ParserNode {
                                    expr: ParserExpr::RepExact(Box::new(node), num),
                                    span: start.span(&pair.as_span().end_pos()),
                                }
                            }
                            Rule::repeat_min => {
                                let mut inner = pair.clone().into_inner();

                                inner.next().unwrap(); // opening_brace

                                let min_number = inner.next().unwrap();
                                let min = if let Ok(min) = min_number.as_str().parse::<u32>() {
                                    min
                                } else {
                                    return Err(vec![Error::new(
                                        ErrorVariant::CustomError {
                                            message: "number cannot overflow u32".to_owned(),
                                        },
                                        vec![LocatedError::new_from_span(min_number.as_span())],
                                    )]);
                                };

                                let start = node.span.start_pos();
                                ParserNode {
                                    expr: ParserExpr::RepMin(Box::new(node), min),
                                    span: start.span(&pair.as_span().end_pos()),
                                }
                            }
                            Rule::repeat_max => {
                                let mut inner = pair.clone().into_inner();

                                inner.next().unwrap(); // opening_brace
                                inner.next().unwrap(); // comma

                                let max_number = inner.next().unwrap();
                                let max = if let Ok(max) = max_number.as_str().parse::<u32>() {
                                    max
                                } else {
                                    return Err(vec![Error::new(
                                        ErrorVariant::CustomError {
                                            message: "number cannot overflow u32".to_owned(),
                                        },
                                        vec![LocatedError::new_from_span(max_number.as_span())],
                                    )]);
                                };

                                if max == 0 {
                                    let error: Error<Rule> = Error::new(
                                        ErrorVariant::CustomError {
                                            message: "cannot repeat 0 times".to_owned(),
                                        },
                                        vec![LocatedError::new_from_span(max_number.as_span())],
                                    );

                                    return Err(vec![error]);
                                }

                                let start = node.span.start_pos();
                                ParserNode {
                                    expr: ParserExpr::RepMax(Box::new(node), max),
                                    span: start.span(&pair.as_span().end_pos()),
                                }
                            }
                            Rule::repeat_min_max => {
                                let mut inner = pair.clone().into_inner();

                                inner.next().unwrap(); // opening_brace

                                let min_number = inner.next().unwrap();
                                let min = if let Ok(min) = min_number.as_str().parse::<u32>() {
                                    min
                                } else {
                                    return Err(vec![Error::new(
                                        ErrorVariant::CustomError {
                                            message: "number cannot overflow u32".to_owned(),
                                        },
                                        vec![LocatedError::new_from_span(min_number.as_span())],
                                    )]);
                                };

                                inner.next().unwrap(); // comma

                                let max_number = inner.next().unwrap();
                                let max = if let Ok(max) = max_number.as_str().parse::<u32>() {
                                    max
                                } else {
                                    return Err(vec![Error::new(
                                        ErrorVariant::CustomError {
                                            message: "number cannot overflow u32".to_owned(),
                                        },
                                        vec![LocatedError::new_from_span(max_number.as_span())],
                                    )]);
                                };

                                if max == 0 {
                                    let error: Error<Rule> = Error::new(
                                        ErrorVariant::CustomError {
                                            message: "cannot repeat 0 times".to_owned(),
                                        },
                                        vec![LocatedError::new_from_span(max_number.as_span())],
                                    );

                                    return Err(vec![error]);
                                }

                                let start = node.span.start_pos();
                                ParserNode {
                                    expr: ParserExpr::RepMinMax(Box::new(node), min, max),
                                    span: start.span(&pair.as_span().end_pos()),
                                }
                            }
                            Rule::closing_paren => {
                                let start = node.span.start_pos();

                                ParserNode {
                                    expr: node.expr,
                                    span: start.span(&pair.as_span().end_pos()),
                                }
                            }
                            _ => unreachable!(),
                        };

                        Ok(node)
                    },
                )?
            }
        };

        Ok(node)
    }

    let term = |pair: Pair<Rule>| unaries(pair.into_inner().peekable(), climber);
    let infix = |lhs: Result<ParserNode, Vec<Error<Rule>>>,
                 op: Pair<Rule>,
                 rhs: Result<ParserNode, Vec<Error<Rule>>>| match op.as_rule() {
        Rule::sequence_operator => {
            let lhs = lhs?;
            let rhs = rhs?;

            let start = lhs.span.start_pos();
            let end = rhs.span.end_pos();

            Ok(ParserNode {
                expr: ParserExpr::Seq(Box::new(lhs), Box::new(rhs)),
                span: start.span(&end),
            })
        }
        Rule::choice_operator => {
            let lhs = lhs?;
            let rhs = rhs?;

            let start = lhs.span.start_pos();
            let end = rhs.span.end_pos();

            Ok(ParserNode {
                expr: ParserExpr::Choice(Box::new(lhs), Box::new(rhs)),
                span: start.span(&end),
            })
        }
        _ => unreachable!(),
    };

    climber.climb(pairs, term, infix)
}

fn unescape(string: &str) -> Option<String> {
    let mut result = String::new();
    let mut chars = string.chars();

    loop {
        match chars.next() {
            Some('\\') => match chars.next()? {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                'r' => result.push('\r'),
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '0' => result.push('\0'),
                '\'' => result.push('\''),
                'x' => {
                    let string: String = chars.clone().take(2).collect();

                    if string.len() != 2 {
                        return None;
                    }

                    for _ in 0..string.len() {
                        chars.next()?;
                    }

                    let value = u8::from_str_radix(&string, 16).ok()?;

                    result.push(char::from(value));
                }
                'u' => {
                    if chars.next()? != '{' {
                        return None;
                    }

                    let string: String = chars.clone().take_while(|c| *c != '}').collect();

                    if string.len() < 2 || 6 < string.len() {
                        return None;
                    }

                    for _ in 0..string.len() + 1 {
                        chars.next()?;
                    }

                    let value = u32::from_str_radix(&string, 16).ok()?;

                    result.push(char::from_u32(value)?);
                }
                _ => return None,
            },
            Some(c) => result.push(c),
            None => return Some(result),
        };
    }
}
