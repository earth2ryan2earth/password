use crate::jni_wrap::symbols::combination_count;

#[derive(Debug, Clone)]
pub struct CrackParam {
    charset: Box<[char]>,
    min_length: u8,
    max_length: u8,
    total_combos: usize,
    output_contains: String,
}

impl CrackParam {
    pub fn new(
        charset: Box<[char]>,
        min_length: u8,
        max_length: u8,
        output_contains: &str,
    ) -> Self {
        let total_combos = combination_count(&charset, min_length, max_length);

        Self {
            charset,
            min_length,
            max_length,
            total_combos,
            output_contains: String::from(output_contains),
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

    pub fn total_combos(&self) -> usize {
        self.total_combos
    }

    pub fn output_contains(&self) -> &String {
        &self.output_contains
    }
}
