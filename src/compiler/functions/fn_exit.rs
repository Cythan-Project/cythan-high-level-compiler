use crate::compiler::{
    asm::Number,
    error::{CError, CErrorType},
    mir::Mir,
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

pub fn EXIT(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 1 {
        return Err(CError(
            vec![fc.span.clone()],
            CErrorType::WrongNumberOfArgument(1),
        ));
    }
    let k = fc.arguments[0].get_value(ss, state, false)?;
    let tmp = k.to_asm(state)?;
    state.instructions.push(Mir::WriteRegister(Number(0), tmp));
    state.instructions.push(Mir::Stop);
    Ok(None)
}
