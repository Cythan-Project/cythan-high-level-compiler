use std::{collections::HashMap, rc::Rc};

use super::{
    error::{CError, CSpan},
    functions::{
        fn_break::BREAK, fn_continue::CONTINUE, fn_dec::DEC, fn_exit::EXIT, fn_fn::FN,
        fn_get_reg::GET_REG, fn_if0::IF0, fn_inc::INC, fn_include::INCLUDE, fn_let::LET,
        fn_loop::LOOP, fn_set::SET, fn_set_reg::SET_REG,
    },
    parser::function_call::FunctionCall,
    state::State,
    type_defs::Handler,
    variable::CVariable,
};

use crate::compiler::type_defs::Result;

#[derive(Clone, Default)]
pub struct ScopedState {
    pub current_loop: Option<usize>, // (start, end)
    variables: HashMap<String, CVariable>,
    call_graph: Vec<String>,
    functions: HashMap<String, Rc<Handler>>,
    pub return_to: usize,
}

impl ScopedState {
    pub fn new() -> Self {
        let mut k = Self::default();
        k.add_function("exit", EXIT);
        k.add_function("fn", FN);
        k.add_function("set", SET);
        k.add_function("if0", IF0);
        k.add_function("set_reg", SET_REG);
        k.add_function("get_reg", GET_REG);
        k.add_function("loop", LOOP);
        k.add_function("break", BREAK);
        k.add_function("continue", CONTINUE);
        k.add_function("dec", DEC);
        k.add_function("inc", INC);
        k.add_function("include", INCLUDE);
        k.add_function("let", LET);
        //k.add_function("if0", IF0);
        k
    }

    pub fn add_function(
        &mut self,
        name: &str,
        handler: impl Fn(&mut State, &mut ScopedState, &FunctionCall) -> Result<Option<CVariable>>
            + 'static,
    ) {
        self.functions
            .insert(name.to_owned(), Rc::new(Box::new(handler)));
    }
    pub fn execute(&mut self, call: &FunctionCall, state: &mut State) -> Result<Option<CVariable>> {
        self.call_graph.push(call.name.clone());
        self.functions
            .get(&call.name)
            .ok_or_else(|| CError::FunctionNotFound(call.span.clone(), call.name.to_string()))?
            .clone()(state, self, call)
    }
    pub fn get_variable(&self, span: &CSpan, name: &str) -> Result<&CVariable> {
        self.variables
            .get(name)
            .ok_or_else(|| CError::VariableNotFound(span.clone(), name.to_owned()))
    }
    pub fn link_variable(&mut self, name: &str, pos: CVariable) {
        self.variables.insert(name.to_owned(), pos);
    }

    pub fn get_or_declare_variable(
        &mut self,
        name: &str,
        span: &CSpan,
        state: &mut State,
    ) -> CVariable {
        if let Some(e) = self.variables.get(name) {
            e.clone()
        } else {
            let k = state.count();
            self.variables
                .insert(name.to_owned(), CVariable::Value(vec![span.clone()], k));
            CVariable::Value(vec![span.clone()], k)
        }
    }

    pub fn declare_variable(&mut self, name: &str, span: CSpan, state: &mut State) -> usize {
        let k = state.count();
        self.variables
            .insert(name.to_owned(), CVariable::Value(vec![span], k));
        k
    }
}
