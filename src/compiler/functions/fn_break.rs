use crate::compiler::{
    asm::{CompilableInstruction, Label, LabelType},
    error::CError,
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

pub fn BREAK(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if !fc.arguments.is_empty() {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 0));
    }

    state
        .instructions
        .push(CompilableInstruction::Jump(Label::new(
            ss.current_loop
                .ok_or_else(|| CError::InvalidBreakOrContinue(fc.span.clone()))?,
            LabelType::LoopEnd,
        )));

    Ok(None)
}
