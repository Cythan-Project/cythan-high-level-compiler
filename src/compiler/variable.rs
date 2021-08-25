use either::Either;

use super::{
    asm::{Number, Var},
    error::CSpan,
};

#[derive(Clone, Debug)]
pub enum CVariable {
    Value(Vec<CSpan>, usize),
    Number(Vec<CSpan>, u8),
}

impl CVariable {
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
        }
    }
    pub fn to_asm(&self) -> Either<Var, Number> {
        match self {
            CVariable::Value(_, a) => Either::Left((*a).into()),
            CVariable::Number(_, a) => Either::Right(Number(*a)),
        }
    }
    pub fn get_span(&self) -> &[CSpan] {
        match self {
            Self::Value(a, _) | Self::Number(a, _) => a,
        }
    }
    pub fn get_value(&self) -> Option<usize> {
        if let Self::Value(_, a) = self {
            Some(*a)
        } else {
            None
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
