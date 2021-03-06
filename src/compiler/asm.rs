use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::template::Template;

use super::mir::MirState;

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
            CompilableInstruction::Jump(a) => write!(f, "jmp {}", a),
            CompilableInstruction::Label(a) => write!(f, "{}", a),
            CompilableInstruction::If0(a, b) => write!(f, "if ${} {}", a.0, b),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(pub usize, pub LabelType);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum AsmValue {
    Var(Var),
    Number(Number),
}

#[allow(unused)]
impl AsmValue {
    pub fn number(&self) -> Option<Number> {
        if let Self::Number(a) = self {
            Some(a.clone())
        } else {
            None
        }
    }
    pub fn var(&self) -> Option<Var> {
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
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Var(pub usize);

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'v{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Number(pub u8);

impl Label {
    pub fn new(count: usize, t: LabelType) -> Self {
        Self(count, t)
    }
    pub fn alloc(state: &mut MirState, t: LabelType) -> Self {
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

pub fn opt_asm(input: Vec<CompilableInstruction>) -> Vec<CompilableInstruction> {
    if input.is_empty() {
        return vec![];
    }
    std::fs::write(
        "target/before_opt.asm",
        input
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    let in_count = input.len();
    let mut out = Vec::new();
    let mut label_map: HashMap<Label, Label> = HashMap::new();
    let mut in_jump = false;
    for el in input {
        if let CompilableInstruction::Jump(b) = &el {
            in_jump = true;
            loop {
                match out.pop() {
                    Some(CompilableInstruction::Label(a)) => {
                        label_map.insert(a, b.clone());
                    }
                    Some(e) => {
                        out.push(e);
                        break;
                    }
                    _ => break,
                }
            }
            out.push(el);
            continue;
        }
        if in_jump
            && matches!(
                &el,
                CompilableInstruction::Label(_) | &CompilableInstruction::If0(..)
            )
        {
            in_jump = false;
        }
        if in_jump {
            continue;
        }
        out.push(el);
    }
    remap(&mut out, &label_map);
    std::fs::write(
        "target/after_opt.asm",
        out.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    println!(
        "Optimized from {} ASM instructions to {} ASM instructions",
        in_count,
        out.len()
    );
    out
}

fn remap(asm: &mut [CompilableInstruction], amap: &HashMap<Label, Label>) {
    asm.iter_mut().for_each(|i| {
        if let CompilableInstruction::Jump(a)
        | CompilableInstruction::Label(a)
        | CompilableInstruction::If0(.., a) = i
        {
            *a = update(a, amap);
        }
    });
}

fn update(a: &Label, amap: &HashMap<Label, Label>) -> Label {
    match amap.get(a) {
        Some(a) => update(a, amap),
        None => a.clone(),
    }
}
