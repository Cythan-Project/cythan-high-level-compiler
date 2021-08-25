use crate::compiler::{
    asm::{CompilableInstruction},
    error::{CError},
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::{execute_code_block, get_var};

pub fn IF0(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() == 3 {
        let count = state.count();
        let count1 = state.count();
        let k1 = get_var(&fc.arguments[0], state, ss)?;
        state
            .instructions
            .push(CompilableInstruction::If0(k1, count.into()));
        let a = execute_code_block(
            &if let Expression::CodeBlock(_s, e) = &fc.arguments[2] {
                e
            } else {
                return Err(CError::ExpectedBlock(fc.arguments[2].get_span().clone()));
            },
            state,
            ss.clone(),
        )?;
        let mut k = 0;
        if let Some(a) = a {
            k += 1;
            state
                .instructions
                .push(CompilableInstruction::Copy(count.into(), a.to_asm()));
        }
        state
            .instructions
            .push(CompilableInstruction::Jump(count1.into()));
        state
            .instructions
            .push(CompilableInstruction::Label(count.into()));
        let b = execute_code_block(
            &if let Expression::CodeBlock(_s, e) = &fc.arguments[1] {
                e
            } else {
                return Err(CError::ExpectedBlock(fc.arguments[1].get_span().clone()));
            },
            state,
            ss.clone(),
        )?;
        state
            .instructions
            .push(CompilableInstruction::Label(count1.into()));
        if let Some(a) = b {
            k += 1;
            state
                .instructions
                .push(CompilableInstruction::Copy(count.into(), a.to_asm()));
        }

        if k == 2 {
            Ok(Some(CVariable::Value(vec![fc.span.clone()], count)))
        } else {
            Ok(None)
        }
    } else if fc.arguments.len() == 2 {
        let k1 = get_var(&fc.arguments[0], state, ss)?;
        let count = state.count();
        let count1 = state.count();
        state
            .instructions
            .push(CompilableInstruction::If0(k1, count.into()));
        state
            .instructions
            .push(CompilableInstruction::Jump(count1.into()));

        state
            .instructions
            .push(CompilableInstruction::Label(count.into()));
        execute_code_block(
            &if let Expression::CodeBlock(_s, e) = &fc.arguments[1] {
                e
            } else {
                return Err(CError::ExpectedBlock(fc.arguments[1].get_span().clone()));
            },
            state,
            ss.clone(),
        )?;
        state
            .instructions
            .push(CompilableInstruction::Label(count1.into()));
        Ok(None)
    } else {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
}
