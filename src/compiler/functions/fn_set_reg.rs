use crate::compiler::{
    asm::Number, error::CError, parser::function_call::FunctionCall, scope::ScopedState,
    state::State, type_defs::Result, variable::CVariable,
};

pub fn SET_REG(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 2 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
    let k1: Number = fc.arguments[0]
        .get_asm_value(ss, state, false)?
        .number()
        .ok_or_else(|| CError::ExpectedNumber(fc.arguments[0].get_span().clone()))?;
    let k2 = fc.arguments[1].get_value(ss, state, false)?;
    state.set_reg(k1, k2.to_asm());
    Ok(None)
}
