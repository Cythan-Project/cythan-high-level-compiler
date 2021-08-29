use std::{fmt::Display, rc::Rc};

use pest::{
    error::{Error, ErrorVariant, LocatedError},
    Span,
};

use crate::compiler::parser::Rule;

pub struct CError(pub Vec<CSpan>, pub CErrorType);

pub enum CErrorType {
    VariableNotFound(String),
    FunctionNotFound(String),
    ExpectedVariable,
    ExpectedLiteral,
    ExpectedBlock,
    FileNotFound(String),
    ParseFileError(Error<Rule>),
    InvalidNumber,
    InvalidBreakOrContinue,
    ExpectedNumber,
    WrongNumberOfArgument(usize),
    FunctionCallDoesntReturnValue,
    InternalCompilerError(String),
}

impl Display for CErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VariableNotFound( a) => write!(f, "Variable `{}` not found", a),
            Self::FunctionNotFound( a) => write!(f, "Function `{}` not found", a),
            Self::ExpectedVariable => write!(f, "Expected variable"),
            Self::ExpectedNumber => write!(f, "Expected number"),
            Self::InvalidNumber => write!(f, "Invalid number"),
            Self::FunctionCallDoesntReturnValue => {
                write!(f, "This function doesn't return any value")
            }
            Self::WrongNumberOfArgument( a) => {
                write!(f, "Invalid number of argument. Expected {} arguments", a)
            }
            Self::ExpectedLiteral => write!(f, "Expected literal"),
            Self::ExpectedBlock => write!(f, "Expected block"),
            Self::InvalidBreakOrContinue => {
                write!(f, "Can't break or continue outside of a loop")
            }
            Self::FileNotFound( b) => write!(
                f,
                "Can't read `{}` file. Ensure that the path is correct",
                b
            ),
            Self::ParseFileError(_) => unreachable!(),
            Self::InternalCompilerError(a) => write!(f,"This error originated from the CythanV3 compiler and should be reported on https://github.com/Cythan-Project/cythan-high-level-compiler\n\
                    You should include your source code and the following error in the report.\n\
                    {}",a),
        }
    }
}

impl Display for CError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_pest_error())
    }
}

impl CError {
    pub fn chain_errors(mut self, span: &[CSpan]) -> Self {
        self.0.append(&mut span.to_vec());
        self
    }
    pub fn chain(mut self, span: CSpan) -> Self {
        self.0.push(span);
        self
    }

    pub fn as_pest_error(&self) -> pest::error::Error<Rule> {
        match &self.1 {
            CErrorType::ParseFileError(a) => {
                let mut k = a.clone();
                for i in &self.0 {
                    k.locations.insert(
                        0,
                        LocatedError::new_from_span(i.span.clone()).with_path(i.get_filename()),
                    );
                }
                k
            }
            _e => build_error(&self.1.to_string(), &self.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CSpan {
    pub filename: Rc<String>,
    pub span: Span,
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
