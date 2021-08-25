use crate::compiler::{
    asm::{CompilableInstruction, Number},
    error::CError,
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::{as_number, get_value};
pub fn SET_REG(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 2 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
    let k1: u8 = as_number(&fc.arguments[0], ss)?;
    let k2 = get_value(&fc.arguments[1], state, ss)?;
    state
        .instructions
        .push(CompilableInstruction::WriteRegister(
            Number(k1),
            k2.to_asm(),
        ));
    Ok(None)
}
