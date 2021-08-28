use crate::compiler::{
    asm::{Number},
    error::CError,
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::get_value;
pub fn EXIT(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 1 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 1));
    }
    let k = get_value(&fc.arguments[0], state, ss)?;
    state.set_reg(Number(0), k.to_asm());
    state.stop();
    Ok(None)
}
