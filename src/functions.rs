use either::Either;

use crate::{
    asm::{CompilableInstruction, Number, Var},
    error::{CError, CSpan},
    CVariable, Expression, FunctionCall, ScopedState, State,
};

type Result<T> = std::result::Result<T, CError>;

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

pub fn EXIT(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 1 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 1));
    }
    let k = get_value(&fc.arguments[0], state, ss)?;
    state
        .instructions
        .push(CompilableInstruction::WriteRegister(Number(0), k.to_asm()));
    state.instructions.push(CompilableInstruction::Stop);
    Ok(None)
}

pub fn DEC(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 1 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 1));
    }

    let k1: Var = if let Expression::Literal(s, var) = &fc.arguments[0] {
        ss.get_or_declare_variable(var, s.clone(), state)
            .get_value()
            .map(Var::from)
            .ok_or_else(|| CError::ExpectedVariable(s.clone()))?
    } else {
        return Err(CError::ExpectedVariable(fc.arguments[0].get_span().clone()));
    };

    state
        .instructions
        .push(CompilableInstruction::Decrement(k1));

    Ok(None)
}

pub fn INC(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 1 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 1));
    }

    let k1: Var = if let Expression::Literal(s, var) = &fc.arguments[0] {
        ss.get_or_declare_variable(var, s.clone(), state)
            .get_value()
            .map(Var::from)
            .ok_or_else(|| CError::ExpectedVariable(s.clone()))?
    } else {
        return Err(CError::ExpectedVariable(fc.arguments[0].get_span().clone()));
    };

    state
        .instructions
        .push(CompilableInstruction::Increment(k1));

    Ok(None)
}

pub fn CONTINUE(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 0 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 0));
    }

    state.instructions.push(CompilableInstruction::Jump(
        ss.current_loop
            .ok_or_else(|| CError::InvalidBreakOrContinue(fc.span.clone()))?
            .0
            .into(),
    ));

    Ok(None)
}
pub fn BREAK(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 0 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 0));
    }

    state.instructions.push(CompilableInstruction::Jump(
        ss.current_loop
            .ok_or_else(|| CError::InvalidBreakOrContinue(fc.span.clone()))?
            .1
            .into(),
    ));

    Ok(None)
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

pub fn LOOP(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 1 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 1));
    }
    let inside = if let Some(Expression::CodeBlock(s, e)) = fc.arguments.get(0) {
        e
    } else {
        return Err(CError::ExpectedBlock(fc.arguments[0].get_span().clone()));
    };
    let start = state.count();
    let end = state.count();

    state
        .instructions
        .push(CompilableInstruction::Label(start.into()));

    let mut k = ss.clone();

    k.current_loop = Some((start, end));

    execute_code_block(inside, state, k)?;

    state
        .instructions
        .push(CompilableInstruction::Jump(start.into()));
    state
        .instructions
        .push(CompilableInstruction::Label(end.into()));

    Ok(None)
}

pub fn GET_REG(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 2 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
    let k1: Var = if let Expression::Literal(s, var) = &fc.arguments[0] {
        ss.get_or_declare_variable(var, s.clone(), state)
            .get_value()
            .map(Var::from)
            .ok_or_else(|| CError::ExpectedVariable(s.clone()))?
    } else {
        return Err(CError::ExpectedVariable(fc.arguments[0].get_span().clone()));
    };
    let k2 = as_number(&fc.arguments[1], ss)?;
    state
        .instructions
        .push(CompilableInstruction::ReadRegister(k1, Number(k2)));
    Ok(None)
}

pub fn SET_REG(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 2 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
    let k1: u8 = as_number(&fc.arguments[0], ss)?;
    let k2 = get_value(&fc.arguments[1], state, ss)?;
    state
        .instructions
        .push(CompilableInstruction::WriteRegister(
            Number(k1),
            k2.to_asm(),
        ));
    Ok(None)
}

pub fn SET(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 2 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
    let k1: Var = if let Expression::Literal(s, var) = &fc.arguments[0] {
        ss.get_or_declare_variable(var, s.clone(), state)
            .get_value()
            .map(Var::from)
            .ok_or_else(|| CError::ExpectedVariable(s.clone()))?
    } else {
        return Err(CError::ExpectedVariable(fc.arguments[0].get_span().clone()));
    };
    let k2 = get_value(&fc.arguments[1], state, ss)?;
    state
        .instructions
        .push(CompilableInstruction::Copy(k1, k2.to_asm()));
    Ok(None)
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

pub fn LET(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 2 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
    let k1: Var = if let Expression::Literal(s, var) = &fc.arguments[0] {
        ss.declare_variable(var, s.clone(), state).into()
    } else {
        return Err(CError::ExpectedVariable(fc.arguments[0].get_span().clone()));
    };
    let k2 = get_value(&fc.arguments[1], state, ss)?;
    state
        .instructions
        .push(CompilableInstruction::Copy(k1, k2.to_asm()));
    Ok(None)
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

pub fn INCLUDE(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 1 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 1));
    }
    let fname = if let Expression::Literal(_, n) = &fc.arguments[0] {
        n
    } else {
        return Err(CError::ExpectedLiteral(fc.arguments[0].get_span().clone()));
    };
    crate::execute_file(fname, state, ss, Some(fc.arguments[0].get_span().clone()))?;
    Ok(None)
}

pub fn FN(state: &mut State, ss: &mut ScopedState, fc: &FunctionCall) -> Result<Option<CVariable>> {
    let g = fc.arguments.len();
    if fc.arguments.len() < 2 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
    let fname = if let Expression::Literal(s, n) = &fc.arguments[0] {
        n
    } else {
        return Err(CError::ExpectedLiteral(fc.arguments[0].get_span().clone()));
    };
    let args: Vec<(String, CSpan)> = fc
        .arguments
        .iter()
        .skip(1)
        .take(g - 2)
        .map(|x| match x {
            Expression::Literal(s, a) => Ok((a.to_owned(), s.clone())),
            a => Err(CError::ExpectedLiteral(a.get_span().clone())),
        })
        .collect::<Result<_>>()?;
    let code = if let Expression::CodeBlock(s, n) = &fc.arguments[g - 1] {
        n.clone()
    } else {
        return Err(CError::ExpectedBlock(
            fc.arguments[g - 1].get_span().clone(),
        ));
    };
    let scos = ss.clone();

    ss.add_function(fname, move |a, b, c| {
        let mut scos = scos.clone();
        let mut vargs = c.arguments.iter();
        for (i, cspan) in &args {
            if i.starts_with("&") {
                match if i.starts_with("&*") {
                    let k = vargs
                        .next()
                        .ok_or_else(|| CError::WrongNumberOfArgument(c.span.clone(), args.len()))?;
                    get_value_and_initialize(k, a, b)?.chain(k.get_span().clone())
                } else {
                    let k = vargs
                        .next()
                        .ok_or_else(|| CError::WrongNumberOfArgument(c.span.clone(), args.len()))?;
                    get_value(k, a, b)?.chain(k.get_span().clone())
                } {
                    CVariable::Value(s, a) => scos.link_variable(
                        if i.starts_with("&*") {
                            &i[2..]
                        } else {
                            &i[1..]
                        },
                        CVariable::Value(s, a).chain(cspan.clone()),
                    ),
                    CVariable::Number(s, ad) => scos.link_variable(
                        if i.starts_with("&*") {
                            &i[2..]
                        } else {
                            &i[1..]
                        },
                        CVariable::Number(s, ad).chain(cspan.clone()),
                    ),
                }
            } else {
                SET1(
                    a,
                    &mut scos,
                    b,
                    i,
                    vargs
                        .next()
                        .ok_or_else(|| CError::WrongNumberOfArgument(c.span.clone(), args.len()))?,
                    cspan.clone(),
                )?;
            };
        }
        execute_code_block(&code, a, scos)
    });
    Ok(None)
}

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
            &if let Expression::CodeBlock(s, e) = &fc.arguments[2] {
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
            &if let Expression::CodeBlock(s, e) = &fc.arguments[1] {
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
            &if let Expression::CodeBlock(s, e) = &fc.arguments[1] {
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
