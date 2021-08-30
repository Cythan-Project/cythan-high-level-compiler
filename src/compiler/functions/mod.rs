#![allow(non_snake_case)]

pub mod fn_break;
pub mod fn_continue;
pub mod fn_dec;
pub mod fn_exit;
pub mod fn_fn;
pub mod fn_get_reg;
pub mod fn_if0;
pub mod fn_inc;
pub mod fn_include;
pub mod fn_let;
pub mod fn_loop;
pub mod fn_set;
pub mod fn_set_reg;

use crate::compiler::{
    asm::Var, error::CSpan, parser::expression::Expression, scope::ScopedState, state::State,
    type_defs::Result,
};

use super::{mir::Mir, variable::CVariable};

pub fn set_variable_to_expression(
    state: &mut State,
    ss: &mut ScopedState,
    ss1: &mut ScopedState,
    var: &str,
    fc: &Expression,
    span: CSpan,
    declare: bool,
) -> Result<()> {
    let k1: Var = if declare {
        ss.declare_variable(var, span, state).into()
    } else {
        ss.get_or_declare_variable(var, &span, state)
            .as_var(state)?
    }; // Changed to replace var
    let k2 = fc.get_value(ss1, state, false)?;
    let tmp = k2.to_asm(state)?;
    state.instructions.push(Mir::Copy(k1, tmp));
    Ok(())
}

pub fn set_variable_to_expression_ref(
    ss: &mut ScopedState,
    var: &str,
    fc: Expression,
    expr_state: ScopedState,
    span: CSpan,
) -> Result<()> {
    ss.declare_cvar(
        var,
        CVariable::ExpressionRef(vec![span], Box::new(fc), expr_state),
    ); // Changed to replace var
    Ok(())
}

pub fn set_variable(
    state: &mut State,
    ss: &mut ScopedState,
    var: &str,
    fc: &Expression,
    span: CSpan,
    declare: bool,
) -> Result<()> {
    let k1: Var = if declare {
        ss.declare_variable(var, span, state).into()
    } else {
        ss.get_or_declare_variable(var, &span, state)
            .as_var(state)?
    }; // Changed to replace var
    let k2 = fc.get_value(ss, state, false)?;
    let tmp = k2.to_asm(state)?;
    state.instructions.push(Mir::Copy(k1, tmp));
    Ok(())
}
