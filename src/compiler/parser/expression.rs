use crate::compiler::error::CSpan;

use super::function_call::FunctionCall;

#[derive(Debug, Clone)]
pub enum Expression {
    FunctionCall(CSpan, FunctionCall),
    CodeBlock(CSpan, Vec<Expression>),
    Literal(CSpan, String),
    Number(CSpan, u8),
}

impl Expression {
    pub fn get_span(&self) -> &CSpan {
        match self {
            Expression::FunctionCall(a, _)
            | Expression::CodeBlock(a, _)
            | Expression::Literal(a, _)
            | Expression::Number(a, _) => a,
        }
    }
}
