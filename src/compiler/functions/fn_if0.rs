use crate::compiler::{
    asm::{Label, LabelType},
    error::{CError, CErrorType},
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

pub fn IF0(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() == 3 {
        let count = state.count();
        let k1 = fc.arguments[0].as_var(ss, state, false)?;
        state.if0(k1, Label::new(count, LabelType::IfStart));
        let a = fc.arguments[2]
            .get_codeblock()?
            .1
            .execute(state, ss.clone())?;
        let mut k = 0;
        if let Some(a) = a {
            k += 1;
            let tmp = a.to_asm(state)?;
            state.copy(count.into(), tmp);
        }
        state.jump(Label::new(count, LabelType::IfEnd));
        state.label(Label::new(count, LabelType::IfStart));
        let b = fc.arguments[1]
            .get_codeblock()?
            .1
            .execute(state, ss.clone())?;
        state.label(Label::new(count, LabelType::IfEnd));
        if let Some(a) = b {
            k += 1;
            let tmp = a.to_asm(state)?;
            state.copy(count.into(), tmp);
        }

        if k == 2 {
            Ok(Some(CVariable::Value(vec![fc.span.clone()], count)))
        } else {
            Ok(None)
        }
    } else if fc.arguments.len() == 2 {
        let k1 = fc.arguments[0].as_var(ss, state, false)?;
        let count = state.count();
        state.if0(k1, Label::new(count, LabelType::IfStart));
        state.jump(Label::new(count, LabelType::IfEnd));

        state.label(Label::new(count, LabelType::IfStart));
        fc.arguments[1]
            .get_codeblock()?
            .1
            .execute(state, ss.clone())?;
        state.label(Label::new(count, LabelType::IfEnd));
        Ok(None)
    } else {
        Err(CError(
            vec![fc.span.clone()],
            CErrorType::WrongNumberOfArgument(2),
        ))
    }
}
