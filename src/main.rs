extern crate pest;
#[macro_use]
extern crate pest_derive;

mod asm;
mod template;

mod error;
mod functions;

use asm::{CompilableInstruction, Number, Var};

use either::Either;
use error::{CError, CSpan};
use functions::*;
use pest::{iterators::Pair, Parser};
use template::Template;

type Result<T> = std::result::Result<T, CError>;

#[derive(Parser)]
#[grammar = "gramar.pest"]
pub struct INIParser;

use std::{collections::HashMap, fs, rc::Rc};

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub span: CSpan,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    FunctionCall(CSpan, FunctionCall),
    CodeBlock(CSpan, Vec<Expression>),
    Literal(CSpan, String),
    Number(CSpan, u8),
}

impl Expression {
    pub fn get_span(&self) -> &CSpan {
        match self {
            Expression::FunctionCall(a, _)
            | Expression::CodeBlock(a, _)
            | Expression::Literal(a, _)
            | Expression::Number(a, _) => a,
        }
    }
}

trait Parse: Sized {
    fn from_pairs(pair: Pair<Rule>, file: &Rc<String>) -> Result<Self>;
}

impl Parse for FunctionCall {
    fn from_pairs(pair: Pair<Rule>, file: &Rc<String>) -> Result<Self> {
        match pair.as_rule() {
            Rule::function_call => {
                let span = pair.as_span();
                let mut i = pair.into_inner();
                Ok(Self {
                    name: i.next().unwrap().as_str().trim().to_owned(),
                    arguments: i
                        .map(|x| Expression::from_pairs(x, file))
                        .collect::<Result<Vec<_>>>()?,
                    span: CSpan::new(file.clone(), span),
                })
            }
            _ => unreachable!(),
        }
    }
}

impl Parse for Expression {
    fn from_pairs(pair: Pair<Rule>, file: &Rc<String>) -> Result<Self> {
        match pair.as_rule() {
            Rule::simple => Self::from_pairs(pair.into_inner().next().unwrap(), file),
            Rule::opera1 => {
                let span = pair.as_span();
                let mut i = pair.into_inner();
                let operator = i.next().unwrap();
                let operator = operator.as_str().trim();
                let expr = Self::from_pairs(i.next().unwrap(), file)?;
                Ok(Self::FunctionCall(
                    CSpan::new(file.clone(), span.clone()),
                    FunctionCall {
                        name: format!("{}_unique", operator),
                        arguments: vec![expr],
                        span: CSpan::new(file.clone(), span),
                    },
                ))
            }
            Rule::opera2 => {
                let span = pair.as_span();
                let mut i = pair.into_inner();
                let expr = Self::from_pairs(i.next().unwrap(), file)?;
                let operator = i.next().unwrap();
                let operator = operator.as_str().trim();
                let expr1 = Self::from_pairs(i.next().unwrap(), file)?;
                Ok(Self::FunctionCall(
                    CSpan::new(file.clone(), span.clone()),
                    FunctionCall {
                        name: format!("{}", operator),
                        arguments: vec![expr, expr1],
                        span: CSpan::new(file.clone(), span),
                    },
                ))
            }
            Rule::expression => Self::from_pairs(pair.into_inner().next().unwrap(), file),
            Rule::literal => Ok(Self::Literal(
                CSpan::new(file.clone(), pair.as_span()),
                pair.as_str().trim().to_owned(),
            )),
            Rule::number => Ok(Self::Number(
                CSpan::new(file.clone(), pair.as_span()),
                pair.as_str()
                    .trim()
                    .parse()
                    .map_err(|_| CError::InvalidNumber(CSpan::new(file.clone(), pair.as_span())))?,
            )),
            Rule::function_call => {
                let span = pair.as_span();
                FunctionCall::from_pairs(pair, file)
                    .map(|x| Self::FunctionCall(CSpan::new(file.clone(), span), x))
            }
            Rule::code_block => Ok(Self::CodeBlock(
                CSpan::new(file.clone(), pair.as_span()),
                pair.into_inner()
                    .map(|x| Expression::from_pairs(x, file))
                    .collect::<Result<Vec<_>>>()?,
            )),
            e => unreachable!("{:?}", e),
        }
    }
}

#[derive(Default)]
pub struct State {
    counter: usize,
    pub instructions: Vec<CompilableInstruction>,
}

impl State {
    pub fn count(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
}

#[derive(Clone, Default)]
pub struct ScopedState {
    current_loop: Option<(usize, usize)>, // (start, end)
    variables: HashMap<String, CVariable>,
    call_graph: Vec<String>,
    functions: HashMap<String, Rc<Handler>>,
    return_to: usize,
}

#[derive(Clone, Debug)]
pub enum CVariable {
    Value(Vec<CSpan>, usize),
    Number(Vec<CSpan>, u8),
}

impl CVariable {
    pub fn chain(self, span: CSpan) -> Self {
        match self {
            CVariable::Value(mut a, b) => {
                a.insert(0, span);
                CVariable::Value(a, b)
            }
            CVariable::Number(mut a, b) => {
                a.insert(0, span);
                CVariable::Number(a, b)
            }
        }
    }
    pub fn to_asm(&self) -> Either<Var, Number> {
        match self {
            CVariable::Value(_, a) => Either::Left((*a).into()),
            CVariable::Number(_, a) => Either::Right(Number(*a)),
        }
    }
    pub fn get_span(&self) -> &[CSpan] {
        match self {
            Self::Value(a, _) | Self::Number(a, _) => a,
        }
    }
    pub fn get_value(&self) -> Option<usize> {
        if let Self::Value(_, a) = self {
            Some(*a)
        } else {
            None
        }
    }
    pub fn get_number(&self) -> Option<u8> {
        if let Self::Number(_, a) = self {
            Some(*a)
        } else {
            None
        }
    }
}

impl ScopedState {
    fn new() -> Self {
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
        span: CSpan,
        state: &mut State,
    ) -> CVariable {
        if let Some(e) = self.variables.get(name) {
            e.clone()
        } else {
            let k = state.count();
            self.variables
                .insert(name.to_owned(), CVariable::Value(vec![span.clone()], k));
            CVariable::Value(vec![span], k)
        }
    }

    pub fn declare_variable(&mut self, name: &str, span: CSpan, state: &mut State) -> usize {
        let k = state.count();
        self.variables
            .insert(name.to_owned(), CVariable::Value(vec![span], k));
        k
    }
}

pub type Handler =
    Box<dyn Fn(&mut State, &mut ScopedState, &FunctionCall) -> Result<Option<CVariable>> + 'static>;

fn main() {
    let mut state = State::default();
    let mut scope = ScopedState::new();

    if let Err(e) = execute_file("test.ct1", &mut state, &mut scope, None) {
        println!("{}", e.as_pest_error());
        panic!()
    }

    println!(
        "{}",
        state
            .instructions
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    );

    std::fs::write("out.ct", compile(&state.instructions)).unwrap();

    // ...
}

fn compile(instructions: &[CompilableInstruction]) -> String {
    let mut template = Template::new(include_str!("../template.ct"));
    let mut ctx = asm::Context::default();
    instructions
        .iter()
        .for_each(|x| x.compile(&mut template, &mut ctx));
    template.build()
}

pub fn execute_file(
    file_name: &str,
    state: &mut State,
    scope: &mut ScopedState,
    span: Option<CSpan>,
) -> Result<()> {
    let unparsed_file = Rc::new(match fs::read_to_string(file_name) {
        Ok(a) => a,
        Err(e) => {
            if let Some(span) = span {
                return Err(CError::FileNotFound(span, file_name.to_owned()));
            } else {
                panic!("{}", e);
            }
        }
    });

    let file = match INIParser::parse(Rule::file, unparsed_file) {
        Ok(e) => e,
        Err(e) => {
            return Err(CError::ParseFileError(span, {
                let mut e = e;
                e.locations[0] = e.locations[0].clone().with_path(file_name);
                e
            }))
        }
    } // unwrap the parse result
    .next()
    .unwrap(); // get and unwrap the `file` rule; never fails

    let file1 = Rc::new(file_name.to_owned());

    let expressions = match file
        .into_inner()
        .filter(|x| !matches!(x.as_rule(), Rule::EOI | Rule::WHITESPACE))
        .map(|x| Expression::from_pairs(x, &file1))
        .collect::<Result<Vec<_>>>()
    {
        Ok(e) => e,
        Err(e) => return Err(CError::ParseFileError(span, e.as_pest_error())),
    };

    exe(&expressions, state, scope)
}

fn exe(expressions: &[Expression], state: &mut State, scope: &mut ScopedState) -> Result<()> {
    for e in expressions {
        match e {
            Expression::FunctionCall(s, a) => {
                scope.execute(a, state)?;
            }
            Expression::CodeBlock(s, a) => {
                exe(a, state, scope)?;
            }
            _ => (),
        }
    }
    Ok(())
}
