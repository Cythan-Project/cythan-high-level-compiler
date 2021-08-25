use either::Either;
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
    asm::{CompilableInstruction, Number, Var},
    error::{CError, CSpan},
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

pub fn get_value(expr: &Expression, state: &mut State, ss: &mut ScopedState) -> Result<CVariable> {
    match expr {
        Expression::FunctionCall(s, a) => Ok(ss
            .execute(a, state)?
            .ok_or_else(|| CError::FunctionCallDoesntReturnValue(s.clone()))?),
        Expression::CodeBlock(s, a) => execute_code_block(a, state, ss.clone())?
            .ok_or_else(|| CError::ExpectedVariable(s.clone())),
        Expression::Literal(s, a) => Ok(ss.get_variable(s, a)?.clone()),
        Expression::Number(s, a) => Ok(CVariable::Number(vec![s.clone()], *a)),
    }
}

pub fn get_value_and_initialize(
    expr: &Expression,
    state: &mut State,
    ss: &mut ScopedState,
) -> Result<CVariable> {
    match expr {
        Expression::FunctionCall(s, a) => Ok(ss
            .execute(a, state)?
            .ok_or_else(|| CError::FunctionCallDoesntReturnValue(s.clone()))?),
        Expression::CodeBlock(s, a) => execute_code_block(a, state, ss.clone())?
            .ok_or_else(|| CError::ExpectedVariable(s.clone())),
        Expression::Literal(s, a) => Ok(ss.get_or_declare_variable(a, s.clone(), state)),
        Expression::Number(s, a) => Ok(CVariable::Number(vec![s.clone()], *a)),
    }
}

pub fn get_var(expr: &Expression, state: &mut State, ss: &mut ScopedState) -> Result<Var> {
    match expr {
        Expression::FunctionCall(s, a) => Ok(ss
            .execute(a, state)?
            .ok_or_else(|| CError::FunctionCallDoesntReturnValue(s.clone()))?
            .get_value()
            .ok_or_else(|| CError::ExpectedVariable(s.clone()))?
            .into()),
        Expression::CodeBlock(s, a) => execute_code_block(a, state, ss.clone())?
            .map(|x| x.get_value())
            .flatten()
            .map(|x| x.into())
            .ok_or_else(|| CError::ExpectedVariable(s.clone())),
        Expression::Number(s, _) => Err(CError::ExpectedVariable(s.clone())),
        Expression::Literal(s, a) => Ok(ss
            .get_variable(s, a)?
            .get_value()
            .ok_or_else(|| CError::ExpectedVariable(s.clone()))?
            .into()),
    }
}

pub fn as_number(expr: &Expression, ss: &ScopedState) -> Result<u8> {
    match expr {
        Expression::Number(s, a) => Ok(*a),
        Expression::Literal(s, a) => match ss.get_variable(s, a)? {
            CVariable::Number(_, a) => Ok(*a),
            CVariable::Value(s1, a) => {
                let mut trace = s1.clone();
                trace.insert(0, s.clone());
                Err(CError::ExpectedNumberReferenceFoundVariable(trace))
            }
        },
        e => Err(CError::ExpectedNumber(e.get_span().clone())),
    }
}

pub fn SET1(
    state: &mut State,
    ss: &mut ScopedState,
    ss1: &mut ScopedState,
    var: &str,
    fc: &Expression,
    span: CSpan,
) -> Result<()> {
    let k1: Var = ss.declare_variable(var, span, state).into(); // Changed to replace var
    let k2 = get_value(&fc, state, ss1)?;
    state
        .instructions
        .push(CompilableInstruction::Copy(k1, k2.to_asm()));
    Ok(())
}

pub fn execute_code_block(
    expr: &[Expression],
    state: &mut State,
    mut ss: ScopedState,
) -> Result<Option<CVariable>> {
    let return_var = state.count();
    ss.return_to = return_var;
    let mut k = None;
    for m in expr {
        match m {
            Expression::FunctionCall(s, m) => {
                k = ss.execute(m, state)?;
            }
            Expression::CodeBlock(s, m) => {
                k = execute_code_block(m, state, ss.clone())?;
            }
            Expression::Literal(s, m) => k = Some(ss.get_variable(s, m)?.clone()),
            Expression::Number(s, a) => k = Some(CVariable::Number(vec![s.clone()], *a)),
        }
    }
    Ok(k)
}
