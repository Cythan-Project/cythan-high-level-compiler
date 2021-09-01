use crate::compiler::{
    asm::Var,
    error::{CError, CErrorType},
    mir::{Mir, MirCodeBlock},
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
        let k1 = fc.arguments[0].as_var(ss, state, false)?;
        //state.instructions.push(Mir::If0());
        let mut tmp_state = state.instructions.clone();
        state.instructions = MirCodeBlock(vec![]);
        let a = if let Some(a) = fc.arguments[2]
            .get_codeblock()?
            .1
            .execute(state, ss.clone())?
        {
            Some(a.to_asm(state)?)
        } else {
            None
        };
        let mut if_1 = state.instructions.clone();
        state.instructions = MirCodeBlock(vec![]);

        let b = if let Some(a) = fc.arguments[1]
            .get_codeblock()?
            .1
            .execute(state, ss.clone())?
        {
            Some(a.to_asm(state)?)
        } else {
            None
        };
        let mut if_2 = state.instructions.clone();

        //state.copy(count.into(), tmp);
        let outvar = match (a, b) {
            (Some(a), Some(b)) => {
                let count = state.count();
                if_1.push(Mir::Copy(Var(count), a));
                if_2.push(Mir::Copy(Var(count), b));
                Some(Var(count))
            }
            _ => None,
        };

        tmp_state.push(Mir::If0(k1, if_1, if_2));
        state.instructions = tmp_state;

        Ok(outvar.map(|x| CVariable::Value(vec![fc.span.clone()], x.0)))
    } else if fc.arguments.len() == 2 {
        let k1 = fc.arguments[0].as_var(ss, state, false)?;

        let mut tmp_state = state.instructions.clone();
        state.instructions = MirCodeBlock(vec![]);
        if let Some(e) = fc.arguments[1]
            .get_codeblock()?
            .1
            .execute(state, ss.clone())?
        {
            e.to_asm(state)?;
        }
        tmp_state.push(Mir::If0(
            k1,
            state.instructions.clone(),
            MirCodeBlock(vec![]),
        ));
        state.instructions = tmp_state;
        Ok(None)
    } else {
        Err(CError(
            vec![fc.span.clone()],
            CErrorType::WrongNumberOfArgument(2),
        ))
    }
}
