use crate::compiler::{
    asm::{Var},
    error::CError,
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::get_value;
pub fn SET(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 2 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
    let k1: Var = if let Expression::Literal(s, var) = &fc.arguments[0] {
        ss.get_or_declare_variable(var, s.clone(), state)
            .get_value()
            .map(Var::from)
            .ok_or_else(|| CError::ExpectedVariable(s.clone()))?
    } else {
        return Err(CError::ExpectedVariable(fc.arguments[0].get_span().clone()));
    };
    let k2 = get_value(&fc.arguments[1], state, ss)?;
    state.copy(k1, k2.to_asm());
    Ok(None)
}
