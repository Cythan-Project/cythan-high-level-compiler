use crate::compiler::{
    asm::{Label, LabelType},
    error::CError,
    parser::{expression::Expression, function_call::FunctionCall},
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
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 1));
    }
    let inside = if let Some(Expression::CodeBlock(_s, e)) = fc.arguments.get(0) {
        e
    } else {
        return Err(CError::ExpectedBlock(fc.arguments[0].get_span().clone()));
    };
    let count = state.count();

    state.label(Label::new(count, LabelType::LoopStart));

    let mut k = ss.clone();

    k.current_loop = Some(count);

    inside.execute(state, k)?;

    state.jump(Label::new(count, LabelType::LoopStart));
    state.label(Label::new(count, LabelType::LoopEnd));

    Ok(None)
}
