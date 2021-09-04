use std::fmt::Display;

pub mod optimizer;

use super::asm::{AsmValue, CompilableInstruction, Label, Number, Var, Mapper, LabelType};

#[derive(PartialEq, Clone, Hash)]
pub enum Mir {
    Copy(Var, AsmValue),                  // to, from - from isn't mutated
    JumpingMapper(Var,Vec<usize>,Vec<MirCodeBlock>),                   
    ChangingMapper(Var,Vec<Number>),
    // If0(Var, MirCodeBlock, MirCodeBlock), // Jumps to the label if the thing is equals to 0
    Loop(MirCodeBlock),
    Break,
    Continue,
    Stop,
    ReadRegister(Var, Number),
    WriteRegister(Number, AsmValue),
}

impl Display for Mir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mir::Copy(a, b) => write!(
                f,
                "v{} = {}",
                a.0,
                match b {
                    AsmValue::Var(a) => format!("v{}", a.0),
                    AsmValue::Number(a) => a.0.to_string(),
                }
            ),
            Mir::JumpingMapper(a,table,codes) => write!(f, 
                "JumpingMap 'v{}: {}",a,
                codes
                    .iter()
                    .enumerate()
                    .map(|(i,x)| format!(
                        "{:?} -> {}", 
                        table
                            .iter()
                            .enumerate()
                            .filter(|(i_t,x_t)| &&i == x_t)
                            .map(|(i_t,x_t)| i_t.to_string())
                            .collect::<Vec<String>>()
                            .join(","),
                        format!(" {{\n{} }}",
                            x.0.iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>()
                                .join("\n")))
                        .replace("\n", "\n  "))
                    .collect::<Vec<String>>()
                    .join("\n")
                    .replace("\n", "\n  ")
            ),
            Mir::ChangingMapper(a,table) => write!(f, 
                "ChangingMap 'v{}: {}",a,
                table
                    .iter()
                    .enumerate()
                    .map(|(i,x)| format!("{} -> {}",i,x))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            //  write!(f, "map:{}", table.iter().enumerate().map(|(i,x)|format!("{}>{}",i,x)).collect::<Vec<String>>().join(","))
            /*Mir::Increment(a) => write!(f, "v{}++", a.0),
            Mir::Decrement(a) => write!(f, "v{}--", a.0),
            Mir::If0(a, b, c) => write!(
                f,
                "if v{} {{\n  {}\n}} else {{\n  {}\n}}",
                a.0,
                b.0.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .replace("\n", "\n  "),
                c.0.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .replace("\n", "\n  ")
            ), */
            Mir::Loop(a) => write!(
                f,
                "loop {{\n  {}\n}}",
                a.0.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .replace("\n", "\n  ")
            ),
            Mir::Break => write!(f, "break"),
            Mir::Continue => write!(f, "continue"),
            Mir::Stop => write!(f, "stop"),
            Mir::ReadRegister(a, b) => write!(f, "v{} = @{}", a.0, b.0),
            Mir::WriteRegister(a, b) => write!(
                f,
                "{} <@ {}",
                a.0,
                match b {
                    AsmValue::Var(a) => format!("v{}", a.0),
                    AsmValue::Number(a) => a.0.to_string(),
                }
            ),
        }
    }
}

#[derive(PartialEq, Clone, Hash)]
pub struct MirCodeBlock(pub Vec<Mir>);

impl MirCodeBlock {
    pub fn push(&mut self, mir: Mir) {
        self.0.push(mir);
    }
    pub fn to_asm(&self, state: &mut MirState) -> SkipStatus {
        for i in &self.0 {
            match i.to_asm(state) {
                SkipStatus::None => (),
                e => return e,
            }
        }
        SkipStatus::None
    }
}

#[derive(Default)]
pub struct MirState {
    pub count: usize,
    pub instructions: Vec<CompilableInstruction>,
    loops: Vec<Label>,
}

impl MirState {
    pub fn count(&mut self) -> usize {
        self.count += 1;
        self.count
    }
    pub fn jump(&mut self, label: Label) {
        self.instructions.push(CompilableInstruction::Jump(label));
    }
    pub fn jumping_map(&mut self, label: Label) {
        self.instructions.push(CompilableInstruction::Jump(label));
    }
    /*pub fn dec(&mut self, variable: Var) {
        self.instructions
            .push(CompilableInstruction::Decrement(variable));
    }
    pub fn inc(&mut self, variable: Var) {
        self.instructions
            .push(CompilableInstruction::Increment(variable));
    }
    pub fn if0(&mut self, variable: Var, label: Label) {
        self.instructions
            .push(CompilableInstruction::If0(variable, label));
    }*/
    
    pub fn mapper(&mut self, variable: Var, map: Mapper) {
        self.instructions.push(CompilableInstruction::Mapper(variable,map));
    }
    pub fn copy(&mut self, variable: Var, value: AsmValue) {
        self.instructions
            .push(CompilableInstruction::Copy(variable, value));
    }
    pub fn get_reg(&mut self, variable: Var, reg: Number) {
        self.instructions
            .push(CompilableInstruction::ReadRegister(variable, reg));
    }
    pub fn set_reg(&mut self, reg: Number, value: AsmValue) {
        self.instructions
            .push(CompilableInstruction::WriteRegister(reg, value));
    }
    pub fn stop(&mut self) {
        self.instructions.push(CompilableInstruction::Stop);
    }
    pub fn label(&mut self, label: Label) {
        self.instructions.push(CompilableInstruction::Label(label));
    }
}

pub enum SkipStatus {
    Stoped,
    Continue,
    Break,
    None,
}

impl SkipStatus {
    fn lightest(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::None, _) => Self::None,
            (_, Self::None) => Self::None,
            (Self::Continue, _) => Self::Continue,
            (_, Self::Continue) => Self::Continue,
            (Self::Break, _) => Self::Break,
            (_, Self::Break) => Self::Break,
            _ => Self::Stoped,
        }
    }
}

impl Mir {
    pub fn to_asm(&self, state: &mut MirState) -> SkipStatus {
        match self {
            Mir::Copy(a, b) => {
                if let AsmValue::Var(b) = b {
                    if a == b {
                        return SkipStatus::None;
                    }
                }
                state.copy(a.clone(), b.clone())
            }
            /*Mir::Increment(a) => state.inc(a.clone()),
            Mir::Decrement(a) => state.dec(a.clone()),
            Mir::If0(a, b, c) => {
                if b == c {
                    return b.to_asm(state);
                }
                let end = Label::alloc(state, crate::compiler::asm::LabelType::IfEnd);
                if b.0.is_empty() {
                    state.if0(a.clone(), end.clone());
                    b.to_asm(state);
                    state.label(end);
                } else {
                    let start = end.derive(LabelType::IfStart);
                    state.if0(a.clone(), start.clone());
                    let if1 = c.to_asm(state);
                    state.jump(end.clone());
                    state.label(start);
                    let if2 = b.to_asm(state);
                    state.label(end);
                    return if1.lightest(&if2);
                }
            } */
            Mir::Loop(a) => {
                // If this happens this means the program will do nothing forever.
                if a.0.is_empty() {
                    let looplabel = Label::alloc(state, crate::compiler::asm::LabelType::LoopStart);
                    state.label(looplabel.clone());
                    state.jump(looplabel);
                    return SkipStatus::Stoped;
                }
                let loopstart = Label::alloc(state, crate::compiler::asm::LabelType::LoopStart);
                let loopend = loopstart.derive(crate::compiler::asm::LabelType::LoopEnd);
                state.label(loopstart.clone());
                state.loops.push(loopstart.clone());
                let k = a.to_asm(state);
                state.loops.pop();
                state.jump(loopstart);
                state.label(loopend);
                if matches!(k, SkipStatus::Stoped) {
                    return SkipStatus::Stoped;
                }
            }
            Mir::Break => {
                state.jump(state.loops.last().unwrap().derive(LabelType::LoopEnd)); // TODO: Add error here
                return SkipStatus::Break;
            }
            Mir::Continue => {
                state.jump(state.loops.last().unwrap().derive(LabelType::LoopStart)); // TODO: Add error here
                return SkipStatus::Continue;
            }
            Mir::Stop => {
                state.stop();
                return SkipStatus::Stoped;
            }
            Mir::ReadRegister(a, b) => state.get_reg(a.clone(), b.clone()),
            Mir::WriteRegister(a, b) => state.set_reg(a.clone(), b.clone()),
        }
        SkipStatus::None
    }
}