use crate::compiler::{
    asm::{Label, LabelType},
    error::{CError, CErrorType},
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};
pub fn CONTINUE(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if !fc.arguments.is_empty() {
        return Err(CError(
            vec![fc.span.clone()],
            CErrorType::WrongNumberOfArgument(0),
        ));
    }

    state.jump(Label::new(
        ss.current_loop
            .ok_or_else(|| CError(vec![fc.span.clone()], CErrorType::InvalidBreakOrContinue))?,
        LabelType::LoopStart,
    ));

    Ok(None)
}
