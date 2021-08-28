use crate::compiler::{
    error::{CError, CSpan},
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::{get_value, get_value_and_initialize, set_variable_to_expression};

pub fn FN(
    _state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    let g = fc.arguments.len();
    if fc.arguments.len() < 2 {
        return Err(CError::WrongNumberOfArgument(fc.span.clone(), 2));
    }
    let fname = if let Expression::Literal(_s, n) = &fc.arguments[0] {
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
    let code = if let Expression::CodeBlock(_s, n) = &fc.arguments[g - 1] {
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
            if let Some(i) = i.strip_prefix('&') {
                match if i.starts_with('*') {
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
                    vargs
                        .next()
                        .ok_or_else(|| CError::WrongNumberOfArgument(c.span.clone(), args.len()))?,
                    cspan.clone(),
                )?;
            };
        }
        code.execute(a, scos)
    });
    Ok(None)
}
