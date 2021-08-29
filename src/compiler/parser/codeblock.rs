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
            match m {
                Expression::FunctionCall(_s, m) => {
                    k = ss.execute(m, state)?;
                }
                Expression::CodeBlock(_s, m) => {
                    k = m.execute(state, ss.clone())?;
                }
                Expression::Literal(s, m) => k = Some(ss.get_variable(&[s.clone()], m)?.clone()),
                Expression::Number(s, a) => k = Some(CVariable::Number(vec![s.clone()], *a)),
            }
        }
        Ok(k)
    }
    pub fn execute(&self, state: &mut State, mut ss: ScopedState) -> Result<Option<CVariable>> {
        let return_var = state.count();
        ss.return_to = return_var;
        let mut k = None;
        for m in &self.0 {
            match m {
                Expression::FunctionCall(_s, m) => {
                    k = ss.execute(m, state)?;
                }
                Expression::CodeBlock(_s, m) => {
                    k = m.execute(state, ss.clone())?;
                }
                Expression::Literal(s, m) => k = Some(ss.get_variable(&[s.clone()], m)?.clone()),
                Expression::Number(s, a) => k = Some(CVariable::Number(vec![s.clone()], *a)),
            }
        }
        Ok(k)
    }
}
