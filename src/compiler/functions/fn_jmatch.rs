use crate::compiler::{
    error::{CError, CErrorType},
    parser::{function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::set_variable;

pub fn J_MATCH(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    unimplemented!()
}
