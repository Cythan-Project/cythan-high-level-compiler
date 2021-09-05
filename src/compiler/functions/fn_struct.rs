use std::collections::HashMap;

use crate::compiler::{
    error::{CError, CErrorType},
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::{CVariable, StructRef},
};

pub fn STRUCT(_: &mut State, ss: &mut ScopedState, fc: &FunctionCall) -> Result<Option<CVariable>> {
    if fc.arguments.len() < 2 {
        return Err(CError(
            vec![fc.span.clone()],
            CErrorType::WrongNumberOfArgument(2),
        ));
    }
    let mut args = fc.arguments.iter();
    let (s, var) = args
        .next()
        .unwrap()
        .get_literal()
        .map(|(a, b)| (a.clone(), b.to_owned()))?;

    let fields: Vec<String> = args
        .map(|x| {
            let (span, string) = x.get_literal()?;
            Ok(if let Some(pos) = string.find("..") {
                if let Some(string) = string.strip_prefix('r') {
                    let pos = pos - 1;
                    let first: u8 = string[0..pos]
                        .parse()
                        .map_err(|_| CError(vec![span.clone()], CErrorType::ExpectedNumber))?;
                    let second: u8 = string[(pos + 2)..]
                        .parse()
                        .map_err(|_| CError(vec![span.clone()], CErrorType::ExpectedNumber))?;
                    (first..second).map(|x| x.to_string()).collect()
                } else {
                    vec![string.to_owned()]
                }
            } else {
                vec![string.to_owned()]
            })
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    // TODO: Add ExpressionRef, Ref fields in Structs
    ss.add_function(&var.clone(), move |a, b, c| {
        if c.arguments.len() != fields.len() {
            return Err(CError(
                vec![c.span.clone()],
                CErrorType::WrongNumberOfArgument(fields.len()),
            ));
        }

        Ok(Some(CVariable::Struct(
            vec![c.span.clone()],
            StructRef {
                span: vec![s.clone()],
                name: var.clone(),
                fields: c
                    .arguments
                    .iter()
                    .zip(fields.iter())
                    .map(|(x, y)| Ok((y.clone(), x.get_value(b, a, false)?)))
                    .collect::<Result<HashMap<String, CVariable>>>()?,
            },
        )))
    });

    Ok(None)
}
