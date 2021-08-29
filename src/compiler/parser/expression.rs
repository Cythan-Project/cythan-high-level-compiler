use crate::compiler::{
    asm::{AsmValue, Number, Var},
    error::{CError, CErrorType, CSpan},
    scope::ScopedState,
    state::State,
    variable::CVariable,
};

use super::{codeblock::CodeBlock, function_call::FunctionCall};

use crate::compiler::type_defs::Result;

#[derive(Debug, Clone)]
pub enum Expression {
    FunctionCall(CSpan, FunctionCall),
    CodeBlock(CSpan, CodeBlock),
    Literal(CSpan, String),
    Number(CSpan, u8),
}

impl Expression {
    pub fn get_literal(&self) -> Result<(&CSpan, &String)> {
        if let Self::Literal(a, b) = self {
            Ok((a, b))
        } else {
            Err(CError(
                vec![self.get_span().clone()],
                CErrorType::ExpectedLiteral,
            ))
        }
    }
    pub fn get_codeblock(&self) -> Result<(&CSpan, &CodeBlock)> {
        if let Self::CodeBlock(a, b) = self {
            Ok((a, b))
        } else {
            Err(CError(
                vec![self.get_span().clone()],
                CErrorType::ExpectedBlock,
            ))
        }
    }

    pub fn get_var(&self, ss: &mut ScopedState, state: &mut State, declare: bool) -> Result<Var> {
        match self.get_value(ss, state, declare)? {
            CVariable::Value(_, a) => Ok(Var(a)),
            CVariable::Number(a, _b) => Err(CError(a, CErrorType::ExpectedVariable)),
        }
    }

    pub fn as_var(&self, ss: &mut ScopedState, state: &mut State, declare: bool) -> Result<Var> {
        self.get_asm_value(ss, state, declare)?
            .var()
            .ok_or_else(|| CError(vec![self.get_span().clone()], CErrorType::ExpectedVariable))
    }

    pub fn as_number(
        &self,
        ss: &mut ScopedState,
        state: &mut State,
        declare: bool,
    ) -> Result<Number> {
        self.get_asm_value(ss, state, declare)?
            .number()
            .ok_or_else(|| CError(vec![self.get_span().clone()], CErrorType::ExpectedNumber))
    }

    pub fn get_asm_value(
        &self,
        ss: &mut ScopedState,
        state: &mut State,
        declare: bool,
    ) -> Result<AsmValue> {
        Ok(match self.get_value(ss, state, declare)? {
            CVariable::Value(_, a) => AsmValue::Var(Var(a)),
            CVariable::Number(_a, b) => AsmValue::Number(Number(b)),
        })
    }

    pub fn get_value(
        &self,
        ss: &mut ScopedState,
        state: &mut State,
        declare: bool,
    ) -> Result<CVariable> {
        Ok(match self {
            Expression::FunctionCall(s, a) => ss.execute(a, state)?.ok_or_else(|| {
                CError(vec![s.clone()], CErrorType::FunctionCallDoesntReturnValue)
            })?,
            Expression::CodeBlock(s, a) => a
                .execute(state, ss.clone())?
                .ok_or_else(|| CError(vec![s.clone()], CErrorType::ExpectedVariable))?,
            Expression::Literal(s, a) => {
                if declare {
                    ss.get_or_declare_variable(a, s, state)
                } else {
                    ss.get_variable(&[s.clone()], a)?.clone()
                }
            }
            Expression::Number(s, a) => CVariable::Number(vec![s.clone()], *a),
        })
    }

    pub fn get_span(&self) -> &CSpan {
        match self {
            Expression::FunctionCall(a, _)
            | Expression::CodeBlock(a, _)
            | Expression::Literal(a, _)
            | Expression::Number(a, _) => a,
        }
    }
}
