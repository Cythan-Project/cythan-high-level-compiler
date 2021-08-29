use crate::compiler::{
    error::{CError, CErrorType},
    parser::{function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::set_variable;

pub fn SET(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 2 {
        return Err(CError(
            vec![fc.span.clone()],
            CErrorType::WrongNumberOfArgument(2),
        ));
    }
    let (s, var) = fc.arguments[0].get_literal()?;
    set_variable(state, ss, var, &fc.arguments[1], s.clone(), false)?;

    Ok(None)
}
