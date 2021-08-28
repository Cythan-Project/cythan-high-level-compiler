use crate::compiler::{
    asm::{Number, Var},
    error::CError,
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::as_number;

pub fn GET_REG(
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
    let k2 = as_number(&fc.arguments[1], ss)?;
    state.get_reg(k1, Number(k2));
    Ok(None)
}
