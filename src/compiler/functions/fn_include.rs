use std::path::Path;

use crate::compiler::{
    error::{CError, CErrorType},
    parser::function_call::FunctionCall,
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
        return Err(CError(
            vec![fc.span.clone()],
            CErrorType::WrongNumberOfArgument(1),
        ));
    }
    let k = fc.span.get_filename();
    let (span, fname) = fc.arguments[0].get_literal()?;
    let mut path = Path::new(k).to_path_buf();
    path.pop();
    crate::execute_file(
        path.join(fname).to_str().unwrap(),
        state,
        ss,
        vec![span.clone()],
    )?;
    Ok(None)
}
