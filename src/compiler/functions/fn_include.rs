use crate::compiler::{
    error::{CError},
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};
pub fn INCLUDE(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 1 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 1));
    }
    let fname = if let Expression::Literal(_, n) = &fc.arguments[0] {
        n
    } else {
        return Err(CError::ExpectedLiteral(fc.arguments[0].get_span().clone()));
    };
    crate::execute_file(fname, state, ss, Some(fc.arguments[0].get_span().clone()))?;
    Ok(None)
}
