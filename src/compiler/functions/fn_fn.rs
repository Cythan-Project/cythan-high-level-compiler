use crate::compiler::{
    error::{CError, CErrorType, CSpan},
    parser::{expression::Expression, function_call::FunctionCall},
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

use super::{set_variable_to_expression, set_variable_to_expression_ref};

enum FnArgument {
    Reference(String, CSpan),
    Copy(String, CSpan),
    DefineReference(String, CSpan),
    ExpressionRef(String, CSpan),
}

impl FnArgument {
    fn new(string: &str, span: CSpan) -> Self {
        if let Some(string) = string.strip_prefix('&') {
            if let Some(string) = string.strip_prefix('*') {
                Self::DefineReference(string.to_owned(), span)
            } else {
                Self::Reference(string.to_owned(), span)
            }
        } else if let Some(string) = string.strip_prefix('$') {
            Self::ExpressionRef(string.to_owned(), span)
        } else {
            Self::Copy(string.to_owned(), span)
        }
    }

    fn execute(
        &self,
        input: &Expression,
        function_scope: &mut ScopedState,
        caller_scope: &mut ScopedState,
        state: &mut State,
    ) -> Result<()> {
        match self {
            Self::Copy(m, n) => set_variable_to_expression(
                state,
                function_scope,
                caller_scope,
                m,
                input,
                n.clone(),
                false,
            )?,
            Self::ExpressionRef(m, n) => set_variable_to_expression_ref(
                function_scope,
                m,
                input.clone(),
                caller_scope.clone(),
                n.clone(),
            )?,
            Self::DefineReference(m, n) | Self::Reference(m, n) => {
                function_scope.link_variable(
                    m,
                    input
                        .get_value(
                            caller_scope,
                            state,
                            matches!(self, FnArgument::DefineReference(..)),
                        )?
                        .chain(n.clone()),
                );
            }
        }
        Ok(())
    }
}

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
    let args: Vec<FnArgument> = fc
        .arguments
        .iter()
        .skip(1)
        .take(g - 2)
        .map(|x| x.get_literal().map(|(a, b)| FnArgument::new(b, a.clone())))
        .collect::<Result<_>>()?;
    let code = fc.arguments[g - 1].get_codeblock()?.1.clone();
    let scos = ss.clone();

    ss.add_function(fname, move |a, b, c| {
        let mut scos = scos.clone();
        let mut vargs = c.arguments.iter();
        for arg in &args {
            arg.execute(
                vargs.next().ok_or_else(|| {
                    CError(
                        vec![c.span.clone()],
                        CErrorType::WrongNumberOfArgument(args.len()),
                    )
                })?,
                &mut scos,
                b,
                a,
            )?;
        }
        code.execute(a, scos)
    });
    Ok(None)
}
