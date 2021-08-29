use crate::compiler::{
    asm::{Label, LabelType},
    error::{CError, CErrorType},
    parser::{function_call::FunctionCall},
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
    let inside = fc.arguments[0].get_codeblock()?.1;
    let count = state.count();

    state.label(Label::new(count, LabelType::LoopStart));

    let mut k = ss.clone();

    k.current_loop = Some(count);

    inside.execute(state, k)?;

    state.jump(Label::new(count, LabelType::LoopStart));
    state.label(Label::new(count, LabelType::LoopEnd));

    Ok(None)
}
