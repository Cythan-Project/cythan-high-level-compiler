use std::collections::HashMap;

use super::{
    asm::{AsmValue, Number, Var},
    error::{CError, CErrorType, CSpan},
    parser::expression::Expression,
    scope::ScopedState,
    state::State,
};

use crate::compiler::type_defs::Result;

#[derive(Clone)]
pub enum CVariable {
    Value(Vec<CSpan>, usize),
    Number(Vec<CSpan>, u8),
    Struct(Vec<CSpan>, StructRef),
    ExpressionRef(Vec<CSpan>, Box<Expression>, ScopedState),
}

#[derive(Clone)]
pub struct StructRef {
    pub span: Vec<CSpan>,
    pub name: String,
    pub fields: HashMap<String, CVariable>,
}

impl CVariable {
    pub fn unroll(&self, state: &mut State) -> Result<Option<Self>> {
        Ok(match self {
            Self::ExpressionRef(_, b, c) => match b.execute(&mut c.clone(), state)? {
                Some(e) => return e.unroll(state),
                None => None,
            },
            e => Some(e.clone()),
        })
    }
    pub fn chain(self, span: CSpan) -> Self {
        match self {
            CVariable::Value(mut a, b) => {
                a.insert(0, span);
                CVariable::Value(a, b)
            }
            CVariable::Number(mut a, b) => {
                a.insert(0, span);
                CVariable::Number(a, b)
            }
            CVariable::ExpressionRef(mut a, b, c) => {
                a.insert(0, span);
                CVariable::ExpressionRef(a, b, c)
            }
            CVariable::Struct(mut a, b) => {
                a.insert(0, span);
                CVariable::Struct(a, b)
            }
        }
    }
    pub fn to_asm(&self, state: &mut State) -> Result<AsmValue> {
        match self {
            CVariable::Value(_, a) => Ok(AsmValue::Var((*a).into())),
            CVariable::Number(_, a) => Ok(AsmValue::Number(Number(*a))),
            CVariable::ExpressionRef(_, a, b) => a.get_asm_value(&mut b.clone(), state, false),
            CVariable::Struct(a, b) => Err(CError(
                a.clone(),
                CErrorType::StructUsedAsVariableInInvalidContext(b.name.to_owned()),
            )),
        }
    }

    pub fn as_var(&self, state: &mut State) -> Result<Var> {
        self.to_asm(state)?
            .var()
            .ok_or_else(|| CError(self.get_span().to_vec(), CErrorType::ExpectedVariable))
    }

    pub fn get_span(&self) -> &[CSpan] {
        match self {
            Self::Value(a, _)
            | Self::Number(a, _)
            | Self::ExpressionRef(a, ..)
            | Self::Struct(a, ..) => a,
        }
    }

    pub fn get_number(&self) -> Option<u8> {
        if let Self::Number(_, a) = self {
            Some(*a)
        } else {
            None
        }
    }
}
