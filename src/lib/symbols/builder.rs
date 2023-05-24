use std::collections::BTreeSet;

use super::{DIGITS, LATIN_LC, LATIN_UC};

/// This module provides a shorthand to build a charset based on characters contained in the library.
/// It is completely optional, & thus possible to create your own custom charsets to pass in.
#[derive(Default, Debug)]
pub struct Builder {
    // A B-Tree Set allows reproducible runs, as the order of the symbols is fixed.
    // No effect on runtime performance.
    chars: BTreeSet<char>,
}

impl Builder {
    /// Creates a new instance of an empty Builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Boxes the charset
    pub fn build(self) -> Box<[char]> {
        if self.chars.is_empty() {
            // TODO Implement Error
            panic!("Empty charset.")
        }
        self.chars
            .into_iter()
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    /// Adds 0..=9
    pub fn with_digits(mut self) -> Self {
        self.chars.extend(&DIGITS);
        self
    }

    /// Adds A..=Z
    pub fn with_latin_uc(mut self) -> Self {
        self.chars.extend(&LATIN_UC);
        self
    }

    /// Adds a..=z
    pub fn with_latin_lc(mut self) -> Self {
        self.chars.extend(&LATIN_LC);
        self
    }

    pub fn with_latin_letters(self) -> Self {
        self.with_latin_uc().with_latin_lc()
    }

    pub fn with_alphanumeric(self) -> Self {
        self.with_latin_letters().with_digits()
    }

    /// Adds a single custom char to the charset
    pub fn with_char(mut self, char: char) -> Self {
        self.chars.insert(char);
        self
    }

    /// Adds a slice of custom characters to the charset
    pub fn with_chars(mut self, chars: &[char]) -> Self {
        self.chars.extend(chars);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }
}

#[cfg(test)]
mod tests_builder {
    use super::*;

    #[test]
    fn test_build() {
        let charset = Builder::new().with_alphanumeric();
        assert!(!charset.is_empty());
    }

    #[test]
    #[should_panic]
    fn test_build_panic() {
        Builder::new().build();
    }
}
