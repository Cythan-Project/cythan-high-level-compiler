use crate::compiler::{
    asm::Var,
    error::{CError, CErrorType},
    parser::{function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

pub fn GET_REG(
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
    let k1: Var = fc.arguments[0].as_var(ss, state, true)?;
    let k2 = fc.arguments[1].as_number(ss, state, false)?;
    state.get_reg(k1, k2);
    Ok(None)
}
