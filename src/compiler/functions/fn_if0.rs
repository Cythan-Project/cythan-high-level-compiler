use crate::compiler::{
    asm::{CompilableInstruction, Label, LabelType},
    error::CError,
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::get_var;

pub fn IF0(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() == 3 {
        let count = state.count();
        let k1 = get_var(&fc.arguments[0], state, ss)?;
        state.instructions.push(CompilableInstruction::If0(
            k1,
            Label::new(count, LabelType::IfStart),
        ));
        let a = if let Expression::CodeBlock(_s, e) = &fc.arguments[2] {
            e.execute(state, ss.clone())?
        } else {
            return Err(CError::ExpectedBlock(fc.arguments[2].get_span().clone()));
        };
        let mut k = 0;
        if let Some(a) = a {
            k += 1;
            state
                .instructions
                .push(CompilableInstruction::Copy(count.into(), a.to_asm()));
        }
        state
            .instructions
            .push(CompilableInstruction::Jump(Label::new(
                count,
                LabelType::IfEnd,
            )));
        state
            .instructions
            .push(CompilableInstruction::Label(Label::new(
                count,
                LabelType::IfStart,
            )));
        let b = if let Expression::CodeBlock(_s, e) = &fc.arguments[1] {
            e.execute(state, ss.clone())?
        } else {
            return Err(CError::ExpectedBlock(fc.arguments[1].get_span().clone()));
        };
        state
            .instructions
            .push(CompilableInstruction::Label(Label::new(
                count,
                LabelType::IfEnd,
            )));
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
        state.instructions.push(CompilableInstruction::If0(
            k1,
            Label::new(count, LabelType::IfStart),
        ));
        state
            .instructions
            .push(CompilableInstruction::Jump(Label::new(
                count,
                LabelType::IfEnd,
            )));

        state
            .instructions
            .push(CompilableInstruction::Label(Label::new(
                count,
                LabelType::IfStart,
            )));
        if let Expression::CodeBlock(_s, e) = &fc.arguments[1] {
            e.execute(state, ss.clone())?;
        } else {
            return Err(CError::ExpectedBlock(fc.arguments[1].get_span().clone()));
        }
        state
            .instructions
            .push(CompilableInstruction::Label(Label::new(
                count,
                LabelType::IfEnd,
            )));
        Ok(None)
    } else {
        Err(CError::WrongNumberOfArgument(fc.span.clone(), 2))
    }
}
