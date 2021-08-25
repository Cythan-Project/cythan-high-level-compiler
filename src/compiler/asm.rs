use std::{borrow::Cow, collections::HashSet, fmt::Display};

use either::Either;

use crate::template::Template;

#[derive(Default)]
pub struct Context {
    variables: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum CompilableInstruction {
    Copy(Var, Either<Var, Number>), // to, from - from isn't mutated
    Increment(Var),                 // in, in is mutated
    Decrement(Var),                 // in, in is mutated
    Jump(Label),                    // Goto a label
    Label(Label),                   // Defines a label
    If0(Var, Label),                // Jumps to the label if the thing is equals to 0
    Stop,
    ReadRegister(Var, Number),
    WriteRegister(Number, Either<Var, Number>),
}

impl CompilableInstruction {
    fn check_compile_var(var: &Var, template: &mut Template, ctx: &mut Context) {
        if !ctx.variables.contains(&var.0) {
            ctx.variables.insert(var.0.to_owned());
            template.add_section("VAR_DEF", Cow::Owned(format!("'v_{}:0", var.0)));
        }
    }

    pub fn compile(&self, template: &mut Template, ctx: &mut Context) {
        match self {
            CompilableInstruction::Copy(a, b) => {
                Self::check_compile_var(a, template, ctx);
                match b {
                    Either::Left(b) => {
                        Self::check_compile_var(b, template, ctx);
                        template.add_code(Cow::Owned(format!("{} {}", b, a)));
                    }
                    Either::Right(b) => {
                        template.add_code(Cow::Owned(format!("'#{} {}", b.0, a)));
                    }
                }
            }
            CompilableInstruction::Increment(a) => {
                Self::check_compile_var(a, template, ctx);
                template.add_code(Cow::Owned(format!("inc({})", a)))
            }
            CompilableInstruction::Decrement(a) => {
                Self::check_compile_var(a, template, ctx);
                template.add_code(Cow::Owned(format!("dec({})", a)))
            }
            CompilableInstruction::Jump(a) => template.add_code(Cow::Owned(format!("jump({})", a))),
            CompilableInstruction::Label(a) => {
                template.add_code(Cow::Owned(format!("{}:no_op", a)))
            }
            CompilableInstruction::If0(a, b) => {
                Self::check_compile_var(a, template, ctx);
                template.add_code(Cow::Owned(format!("if_0({} {})", a, b)))
            }
            CompilableInstruction::Stop => template.add_code(Cow::Borrowed("stop")),
            CompilableInstruction::ReadRegister(a, b) => {
                template.add_code(Cow::Owned(format!("'#int_{} {}", b.0, a)));
            }
            CompilableInstruction::WriteRegister(a, b) => match b {
                Either::Left(b) => {
                    Self::check_compile_var(b, template, ctx);
                    template.add_code(Cow::Owned(format!("{} '#int_{}", b, a.0)));
                }
                Either::Right(b) => {
                    template.add_code(Cow::Owned(format!("'#{} '#int_{}", b.0, a.0)));
                }
            },
        }
    }
}

impl Display for CompilableInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilableInstruction::Copy(a, b) => write!(
                f,
                "${} = {}",
                a.0,
                match b {
                    Either::Left(a) => format!("${}", a.0),
                    Either::Right(a) => a.0.to_string(),
                }
            ),
            CompilableInstruction::Increment(a) => write!(f, "${}++", a.0,),
            CompilableInstruction::Decrement(a) => write!(f, "${}--", a.0,),
            CompilableInstruction::Jump(a) => write!(f, "jmp '{}", a.0),
            CompilableInstruction::Label(a) => write!(f, "'{}", a.0),
            CompilableInstruction::If0(a, b) => write!(f, "if ${} '{}", a.0, b.0),
            CompilableInstruction::Stop => write!(f, "stop"),
            CompilableInstruction::ReadRegister(a, b) => write!(f, "${} = @{}", a.0, b.0),
            CompilableInstruction::WriteRegister(a, b) => write!(
                f,
                "@{} = {}",
                a.0,
                match b {
                    Either::Left(a) => format!("${}", a.0),
                    Either::Right(a) => a.0.to_string(),
                }
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Label(pub String);

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'l_{}", self.0)
    }
}
#[derive(Debug, Clone)]
pub struct Var(pub String);

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'v_{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Number(pub u8);

impl From<usize> for Label {
    fn from(val: usize) -> Self {
        Self(val.to_string())
    }
}
impl From<usize> for Var {
    fn from(val: usize) -> Self {
        Self(val.to_string())
    }
}
