use std::rc::Rc;

use pest::{iterators::Pair, Parser};

use crate::compiler::{
    error::{CError, CErrorType},
    type_defs::Result,
};

use self::expression::Expression;

use super::error::CSpan;

pub mod codeblock;
pub mod expression;
pub mod function_call;
pub mod logic;

#[derive(Parser)]
#[grammar = "./compiler/parser/gramar.pest"]
pub struct CythanParser;

trait Parse: Sized {
    fn from_pairs(pair: Pair<Rule>, file: &Rc<String>) -> Result<Self>;
}

pub fn parse_file(
    file_name: &str,
    file_content: String,
    span: Vec<CSpan>,
) -> Result<Vec<Expression>> {
    let unparsed_file = Rc::new(file_content);

    let file = match CythanParser::parse(Rule::file, unparsed_file) {
        Ok(e) => e,
        Err(e) => {
            return Err(CError(
                span,
                CErrorType::ParseFileError({
                    let mut e = e;
                    e.locations[0] = e.locations[0].clone().with_path(file_name);
                    e
                }),
            ))
        }
    } // unwrap the parse result
    .next()
    .unwrap(); // get and unwrap the `file` rule; never fails

    let file1 = Rc::new(file_name.to_owned());

    match file
        .into_inner()
        .filter(|x| !matches!(x.as_rule(), Rule::EOI | Rule::WHITESPACE))
        .map(|x| Expression::from_pairs(x, &file1))
        .collect::<Result<Vec<_>>>()
    {
        Ok(e) => Ok(e),
        Err(e) => Err(e.chain_errors(&span)),
    }
}
