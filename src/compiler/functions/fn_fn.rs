use crate::compiler::{
    error::{CError, CErrorType, CSpan},
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::set_variable_to_expression;

pub fn FN(
    _state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    let g = fc.arguments.len();
    if fc.arguments.len() < 2 {
        return Err(CError(
            vec![fc.span.clone()],
            CErrorType::WrongNumberOfArgument(2),
        ));
    }
    let fname = fc.arguments[0].get_literal()?.1;
    let args: Vec<(String, CSpan)> = fc
        .arguments
        .iter()
        .skip(1)
        .take(g - 2)
        .map(|x| x.get_literal().map(|(a, b)| (b.clone(), a.clone())))
        .collect::<Result<_>>()?;
    let code = fc.arguments[g - 1].get_codeblock()?.1.clone();
    let scos = ss.clone();

    ss.add_function(fname, move |a, b, c| {
        let mut scos = scos.clone();
        let mut vargs = c.arguments.iter();
        for (i, cspan) in &args {
            if let Some(i) = i.strip_prefix('&') {
                let k = vargs.next().ok_or_else(|| {
                    CError(
                        vec![c.span.clone()],
                        CErrorType::WrongNumberOfArgument(args.len()),
                    )
                })?;
                match k
                    .get_value(b, a, i.starts_with('*'))?
                    .chain(k.get_span().clone())
                {
                    CVariable::Value(s, a) => scos.link_variable(
                        if let Some(e) = i.strip_prefix('*') {
                            e
                        } else {
                            i
                        },
                        CVariable::Value(s, a).chain(cspan.clone()),
                    ),
                    CVariable::Number(s, ad) => scos.link_variable(
                        if let Some(e) = i.strip_prefix('*') {
                            e
                        } else {
                            i
                        },
                        CVariable::Number(s, ad).chain(cspan.clone()),
                    ),
                }
            } else {
                set_variable_to_expression(
                    a,
                    &mut scos,
                    b,
                    i,
                    vargs.next().ok_or_else(|| {
                        CError(
                            vec![c.span.clone()],
                            CErrorType::WrongNumberOfArgument(args.len()),
                        )
                    })?,
                    cspan.clone(),
                    false,
                )?;
            };
        }
        code.execute(a, scos)
    });
    Ok(None)
}
