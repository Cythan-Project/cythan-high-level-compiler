use std::{fmt::Display, rc::Rc};

use pest::{
    error::{Error, ErrorVariant, LocatedError},
    Span,
};

use crate::compiler::parser::Rule;

pub enum CError {
    VariableNotFound(CSpan, String),
    FunctionNotFound(CSpan, String),
    ExpectedVariable(CSpan),
    ExpectedLiteral(CSpan),
    ExpectedBlock(CSpan),
    FileNotFound(CSpan, String),
    ParseFileError(Option<CSpan>, Error<Rule>),
    InvalidNumber(CSpan),
    InvalidBreakOrContinue(CSpan),
    ExpectedNumber(CSpan),
    ExpectedNumberReferenceFoundVariable(Vec<CSpan>),
    WrongNumberOfArgument(CSpan, usize),
    FunctionCallDoesntReturnValue(CSpan),
    InternalCompilerError(String),
}

impl Display for CError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CError::VariableNotFound(_, a) => write!(f, "Variable `{}` not found", a),
            CError::FunctionNotFound(_, a) => write!(f, "Function `{}` not found", a),
            CError::ExpectedVariable(_) => write!(f, "Expected variable"),
            CError::ExpectedNumber(_) => write!(f, "Expected number"),
            CError::InvalidNumber(_) => write!(f, "Invalid number"),
            CError::FunctionCallDoesntReturnValue(_) => {
                write!(f, "This function doesn't return any value")
            }
            CError::WrongNumberOfArgument(_, a) => {
                write!(f, "Invalid number of argument. Expected {} arguments", a)
            }
            CError::ExpectedLiteral(_) => write!(f, "Expected literal"),
            CError::ExpectedBlock(_) => write!(f, "Expected block"),
            CError::InvalidBreakOrContinue(_) => {
                write!(f, "Can't break or continue outside of a loop")
            }
            CError::ExpectedNumberReferenceFoundVariable(_) => {
                write!(f, "Expected number reference but found variable")
            }
            CError::FileNotFound(_, b) => write!(
                f,
                "Can't read `{}` file. Ensure that the path is correct",
                b
            ),
            CError::ParseFileError(_, _) => unreachable!(),
            CError::InternalCompilerError(a) => write!(f,"This error originated from the CythanV3 compiler and should be reported on https://github.com/Cythan-Project/cythan-high-level-compiler\n\
                    You should include your source code and the following error in the report.\n\
                    {}",a),
        }
    }
}

impl CError {
    pub fn get_spans(&self) -> Vec<CSpan> {
        match self {
            CError::VariableNotFound(a, _)
            | CError::FunctionNotFound(a, _)
            | CError::WrongNumberOfArgument(a, _)
            | CError::ExpectedVariable(a)
            | CError::InvalidNumber(a)
            | CError::FileNotFound(a, _)
            | CError::ExpectedLiteral(a)
            | CError::InvalidBreakOrContinue(a)
            | CError::ExpectedBlock(a)
            | CError::FunctionCallDoesntReturnValue(a)
            | CError::ExpectedNumber(a) => vec![a.clone()],
            CError::ExpectedNumberReferenceFoundVariable(a) => a.clone(),
            CError::ParseFileError(_, _) => unreachable!(),
            CError::InternalCompilerError(_) => unreachable!(),
        }
    }

    pub fn display(&self) -> String {
        match self {
            Self::InternalCompilerError(_) => self.to_string(),
            e => e.as_pest_error().to_string(),
        }
    }

    pub fn as_pest_error(&self) -> pest::error::Error<Rule> {
        match self {
            Self::ParseFileError(x, a) => {
                let mut k = a.clone();
                if let Some(x) = x {
                    k.locations.insert(
                        0,
                        LocatedError::new_from_span(x.span.clone()).with_path(&x.filename),
                    );
                }
                k
            }
            _e => build_error(&self.to_string(), &self.get_spans()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CSpan {
    filename: Rc<String>,
    span: Span,
}

impl CSpan {
    pub fn new(filename: Rc<String>, span: Span) -> Self {
        Self { filename, span }
    }

    pub fn get_filename(&self) -> &str {
        self.filename.as_str()
    }
}

fn build_error(message: &str, span: &[CSpan]) -> pest::error::Error<Rule> {
    pest::error::Error::new(
        ErrorVariant::CustomError {
            message: message.to_owned(),
        },
        span.iter()
            .map(|x| LocatedError::new_from_span(x.span.clone()).with_path(&x.filename))
            .collect(),
    )
}
