use super::{
    mir::MirCodeBlock,
};

pub struct State {
    counter: usize,
    pub base: u8,
    pub instructions: MirCodeBlock,
}

impl Default for State {
    fn default() -> Self {
        State {
            counter: 0,
            base: 4,
            instructions: MirCodeBlock(Vec::new()),
        }
    }
}

impl State {
    pub fn count(&mut self) -> usize {
        self.counter += 1;
        self.counter
    }
}
