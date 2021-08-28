use std::rc::Rc;

use pest::iterators::Pair;

use crate::compiler::error::{CError, CSpan};
use crate::compiler::parser::codeblock::CodeBlock;
use crate::compiler::parser::{expression::Expression, Rule};

use crate::compiler::type_defs::Result;

use super::{function_call::FunctionCall, Parse};

impl Parse for FunctionCall {
    fn from_pairs(pair: Pair<Rule>, file: &Rc<String>) -> Result<Self> {
        match pair.as_rule() {
            Rule::function_call => {
                let span = pair.as_span();
                let mut i = pair.into_inner();
                Ok(Self {
                    name: i.next().unwrap().as_str().trim().to_owned(),
                    arguments: i
                        .map(|x| Expression::from_pairs(x, file))
                        .collect::<Result<Vec<_>>>()?,
                    span: CSpan::new(file.clone(), span),
                })
            }
            _ => unreachable!(),
        }
    }
}

impl Parse for Expression {
    fn from_pairs(pair: Pair<Rule>, file: &Rc<String>) -> Result<Self> {
        match pair.as_rule() {
            Rule::simple => Self::from_pairs(pair.into_inner().next().unwrap(), file),
            Rule::opera1 => {
                let span = pair.as_span();
                let mut i = pair.into_inner();
                let operator = i.next().unwrap();
                let operator = operator.as_str().trim();
                let expr = Self::from_pairs(i.next().unwrap(), file)?;
                Ok(Self::FunctionCall(
                    CSpan::new(file.clone(), span.clone()),
                    FunctionCall {
                        name: format!("{}_unique", operator),
                        arguments: vec![expr],
                        span: CSpan::new(file.clone(), span),
                    },
                ))
            }
            Rule::opera2 => {
                let span = pair.as_span();
                let mut i = pair.into_inner();
                let expr = Self::from_pairs(i.next().unwrap(), file)?;
                let operator = i.next().unwrap();
                let operator = operator.as_str().trim();
                let expr1 = Self::from_pairs(i.next().unwrap(), file)?;
                Ok(Self::FunctionCall(
                    CSpan::new(file.clone(), span.clone()),
                    FunctionCall {
                        name: operator.to_string(),
                        arguments: vec![expr, expr1],
                        span: CSpan::new(file.clone(), span),
                    },
                ))
            }
            Rule::expression => Self::from_pairs(pair.into_inner().next().unwrap(), file),
            Rule::literal => Ok(Self::Literal(
                CSpan::new(file.clone(), pair.as_span()),
                pair.as_str().trim().to_owned(),
            )),
            Rule::number => Ok(Self::Number(
                CSpan::new(file.clone(), pair.as_span()),
                pair.as_str()
                    .trim()
                    .parse()
                    .map_err(|_| CError::InvalidNumber(CSpan::new(file.clone(), pair.as_span())))?,
            )),
            Rule::function_call => {
                let span = pair.as_span();
                FunctionCall::from_pairs(pair, file)
                    .map(|x| Self::FunctionCall(CSpan::new(file.clone(), span), x))
            }
            Rule::code_block => Ok(Self::CodeBlock(
                CSpan::new(file.clone(), pair.as_span()),
                CodeBlock(
                    pair.into_inner()
                        .map(|x| Expression::from_pairs(x, file))
                        .collect::<Result<Vec<_>>>()?,
                ),
            )),
            e => unreachable!("{:?}", e),
        }
    }
}
