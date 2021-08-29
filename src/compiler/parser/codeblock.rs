use crate::compiler::{scope::ScopedState, state::State, variable::CVariable};

use super::expression::Expression;

use crate::compiler::type_defs::Result;

#[derive(Clone, Debug)]
pub struct CodeBlock(pub Vec<Expression>);

impl CodeBlock {
    pub fn execute_with_scope(
        &self,
        state: &mut State,
        ss: &mut ScopedState,
    ) -> Result<Option<CVariable>> {
        let return_var = state.count();
        ss.return_to = return_var;
        let mut k = None;
        for m in &self.0 {
            k = m.execute(ss, state)?;
        }
        Ok(k)
    }
    pub fn execute(&self, state: &mut State, mut ss: ScopedState) -> Result<Option<CVariable>> {
        let return_var = state.count();
        ss.return_to = return_var;
        let mut k = None;
        for m in &self.0 {
            k = m.execute(&mut ss, state)?;
        }
        Ok(k)
    }
}
