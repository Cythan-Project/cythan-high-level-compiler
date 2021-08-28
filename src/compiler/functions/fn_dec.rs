use crate::compiler::{
    asm::Var,
    error::CError,
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};
pub fn DEC(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 1 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 1));
    }

    let k1: Var = if let Expression::Literal(s, var) = &fc.arguments[0] {
        ss.get_or_declare_variable(var, s, state)
            .get_value()
            .map(Var::from)
            .ok_or_else(|| CError::ExpectedVariable(s.clone()))?
    } else {
        return Err(CError::ExpectedVariable(fc.arguments[0].get_span().clone()));
    };

    state.dec(k1);

    Ok(None)
}
