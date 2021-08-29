use crate::compiler::{
    asm::Number,
    error::{CError, CErrorType},
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

pub fn SET_REG(
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
    let k1: Number = fc.arguments[0].as_number(ss, state, false)?;
    let k2 = fc.arguments[1].get_value(ss, state, false)?;
    let tmp = k2.to_asm(state)?;
    state.set_reg(k1, tmp);
    Ok(None)
}
