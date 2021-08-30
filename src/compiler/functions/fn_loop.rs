use crate::compiler::{
    error::{CError, CErrorType},
    mir::{Mir, MirCodeBlock},
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

pub fn LOOP(
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

    let mut k = ss.clone();

    let mut tmp_state = state.instructions.clone();
    state.instructions = MirCodeBlock(vec![]);

    k.current_loop = Some(());

    fc.arguments[0].get_codeblock()?.1.execute(state, k)?;

    tmp_state.push(Mir::Loop(state.instructions.clone()));
    state.instructions = tmp_state;

    Ok(None)
}
