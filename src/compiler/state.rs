use super::asm::CompilableInstruction;

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
