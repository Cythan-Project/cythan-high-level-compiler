use crate::compiler::{
    error::CError,
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::{set_variable};

pub fn SET(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 2 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
    if let Expression::Literal(s, var) = &fc.arguments[0] {
        set_variable(state, ss, var, &fc.arguments[1], s.clone(), false)?;
    } else {
        return Err(CError::ExpectedVariable(fc.arguments[0].get_span().clone()));
    };
    Ok(None)
}
