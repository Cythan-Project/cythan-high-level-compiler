use super::{
    error::CError, parser::function_call::FunctionCall, scope::ScopedState, state::State,
    variable::CVariable,
};

pub type Result<T> = std::result::Result<T, CError>;

pub type Handler =
    Box<dyn Fn(&mut State, &mut ScopedState, &FunctionCall) -> Result<Option<CVariable>> + 'static>;
