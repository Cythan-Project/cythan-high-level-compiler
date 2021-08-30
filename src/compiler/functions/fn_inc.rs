use crate::compiler::{
    asm::Var,
    error::{CError, CErrorType},
    mir::Mir,
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};
pub fn INC(
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

    let k1: Var = fc.arguments[0].as_var(ss, state, true)?;
    state.instructions.push(Mir::Increment(k1));

    Ok(None)
}
