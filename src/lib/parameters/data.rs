use crate::parameters::Spawn;
use crate::symbols::combination_count;

#[derive(Debug, Clone)]
pub struct CrackParam<I, S> {
    spawn: Spawn<I, S>,
    charset: Box<[char]>,
    min_length: u8,
    max_length: u8,
    total_combos: usize,
}

impl<I, S> CrackParam<I, S> {
    pub fn new(spawn: Spawn<I, S>, charset: Box<[char]>, min_length: u8, max_length: u8) -> Self {
        let total_combos = combination_count(&charset, min_length, max_length);

        Self {
            charset,
            min_length,
            max_length,
            spawn,
            total_combos,
        }
    }

    pub const fn charset(&self) -> &[char] {
        &self.charset
    }

    pub const fn max_length(&self) -> u8 {
        self.max_length
    }

    pub const fn min_length(&self) -> u8 {
        self.min_length
    }

    pub const fn spawn(&self) -> &Spawn<I, S> {
        &self.spawn
    }

    pub fn total_combos(&self) -> usize {
        self.total_combos
    }
}
