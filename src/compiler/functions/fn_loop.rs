use crate::compiler::{
    asm::CompilableInstruction,
    error::CError,
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::execute_code_block;
pub fn LOOP(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 1 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 1));
    }
    let inside = if let Some(Expression::CodeBlock(_s, e)) = fc.arguments.get(0) {
        e
    } else {
        return Err(CError::ExpectedBlock(fc.arguments[0].get_span().clone()));
    };
    let start = state.count();
    let end = state.count();

    state
        .instructions
        .push(CompilableInstruction::Label(start.into()));

    let mut k = ss.clone();

    k.current_loop = Some((start, end));

    execute_code_block(inside, state, k)?;

    state
        .instructions
        .push(CompilableInstruction::Jump(start.into()));
    state
        .instructions
        .push(CompilableInstruction::Label(end.into()));

    Ok(None)
}
