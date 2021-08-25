use crate::compiler::{
    asm::CompilableInstruction, error::CError, parser::function_call::FunctionCall,
    scope::ScopedState, state::State, type_defs::Result, variable::CVariable,
};
pub fn CONTINUE(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 0 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 0));
    }

    state.instructions.push(CompilableInstruction::Jump(
        ss.current_loop
            .ok_or_else(|| CError::InvalidBreakOrContinue(fc.span.clone()))?
            .0
            .into(),
    ));

    Ok(None)
}
