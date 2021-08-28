use std::{borrow::Cow, collections::HashSet, fmt::Display};

use crate::template::Template;

use super::state::State;

#[derive(Default)]
pub struct Context {
    variables: HashSet<usize>,
}

#[derive(Debug, Clone)]
pub enum CompilableInstruction {
    Copy(Var, AsmValue), // to, from - from isn't mutated
    Increment(Var),      // in, in is mutated
    Decrement(Var),      // in, in is mutated
    Jump(Label),         // Goto a label
    Label(Label),        // Defines a label
    If0(Var, Label),     // Jumps to the label if the thing is equals to 0
    Stop,
    ReadRegister(Var, Number),
    WriteRegister(Number, AsmValue),
}

impl CompilableInstruction {
    fn check_compile_var(var: &Var, template: &mut Template, ctx: &mut Context) {
        if !ctx.variables.contains(&var.0) {
            ctx.variables.insert(var.0);
            template.add_section("VAR_DEF", Cow::Owned(format!("{}:0", var)));
        }
    }

    pub fn compile(&self, template: &mut Template, ctx: &mut Context) {
        match self {
            CompilableInstruction::Copy(a, b) => {
                Self::check_compile_var(a, template, ctx);
                match b {
                    AsmValue::Var(b) => {
                        Self::check_compile_var(b, template, ctx);
                        template.add_code(Cow::Owned(format!("{} {}", b, a)));
                    }
                    AsmValue::Number(b) => {
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
                AsmValue::Var(b) => {
                    Self::check_compile_var(b, template, ctx);
                    template.add_code(Cow::Owned(format!("{} '#int_{}", b, a.0)));
                }
                AsmValue::Number(b) => {
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
                    AsmValue::Var(a) => format!("${}", a.0),
                    AsmValue::Number(a) => a.0.to_string(),
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
                    AsmValue::Var(a) => format!("${}", a.0),
                    AsmValue::Number(a) => a.0.to_string(),
                }
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Label(pub usize, pub LabelType);

#[derive(Debug, Clone)]
pub enum LabelType {
    LoopStart,
    LoopEnd,
    FunctionEnd,
    IfStart,
    ElseStart,
    IfEnd,
}

impl Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LabelType::LoopStart => 'A',
                LabelType::LoopEnd => 'B',
                LabelType::FunctionEnd => 'C',
                LabelType::IfStart => 'D',
                LabelType::ElseStart => 'E',
                LabelType::IfEnd => 'F',
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum AsmValue {
    Var(Var),
    Number(Number),
}

#[allow(unused)]
impl AsmValue {
    fn number(&self) -> Option<Number> {
        if let Self::Number(a) = self {
            Some(a.clone())
        } else {
            None
        }
    }
    fn var(&self) -> Option<Var> {
        if let Self::Var(a) = self {
            Some(a.clone())
        } else {
            None
        }
    }
}

impl From<u8> for AsmValue {
    fn from(a: u8) -> Self {
        AsmValue::Number(Number(a))
    }
}

impl From<usize> for AsmValue {
    fn from(a: usize) -> Self {
        AsmValue::Var(Var(a))
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'l{}{}", self.1, self.0)
    }
}
#[derive(Debug, Clone)]
pub struct Var(pub usize);

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'v{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Number(pub u8);

impl Label {
    pub fn new(count: usize, t: LabelType) -> Self {
        Self(count, t)
    }
    pub fn alloc(state: &mut State, t: LabelType) -> Self {
        Self(state.count(), t)
    }
    pub fn derive(&self, t: LabelType) -> Self {
        Self(self.0, t)
    }
}
impl From<usize> for Var {
    fn from(val: usize) -> Self {
        Self(val)
    }
}
