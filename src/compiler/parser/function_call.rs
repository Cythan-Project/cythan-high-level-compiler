use crate::compiler::error::CSpan;

use super::expression::Expression;

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub span: CSpan,
    pub arguments: Vec<Expression>,
}
