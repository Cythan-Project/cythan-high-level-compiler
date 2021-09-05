use crate::compiler::{
    error::{CError, CErrorType},
    parser::function_call::FunctionCall,
    scope::ScopedState,
    state::State,
    type_defs::Result,
    variable::CVariable,
};

pub fn GET_FIELD(
    state: &mut State,
    ss: &mut ScopedState,
    fc: &FunctionCall,
) -> Result<Option<CVariable>> {
    if fc.arguments.len() != 2 {
        return Err(CError(
            vec![fc.span.clone()],
            CErrorType::WrongNumberOfArgument(2),
        ));
    }
    let (span, field_name) = fc.arguments[1].get_literal()?;
    let k1: CVariable = match fc.arguments[0].get_value(ss, state, false)?.unroll(state)? {
        None => return Err(CError(vec![fc.span.clone()], CErrorType::ExpectedVariable)),
        Some(CVariable::Struct(_, b)) => b
            .fields
            .get(field_name)
            .ok_or_else(|| {
                CError(
                    vec![span.clone()],
                    CErrorType::FieldNotFound(field_name.clone(), b.name.clone()),
                )
            })?
            .clone(),
        Some(e) => return Err(CError(e.get_span().to_vec(), CErrorType::ExpectedStruct)),
    };

    Ok(Some(k1))
}
