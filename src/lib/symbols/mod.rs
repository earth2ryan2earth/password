mod builder;

// Export
pub use builder::Builder;

/// Arabic Numerals
pub static DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// Latin Uppercase Letters
pub static LATIN_UC: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

/// Latin Lowercase Letters
pub static LATIN_LC: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// Calculates the amount of possible permutations given n symbols and m places.
pub fn combination_count(charset: &[char], min_length: u8, max_length: u8) -> usize {
    if min_length > max_length {
        // TODO Substitutes string panic for Errors using crates "error-stack" & "thiserror"
        panic!("min length must be <= max length")
    }
    let mut sum = 0;
    for i in min_length..=max_length {
        sum += charset.len().pow(i.into());
    }
    sum
}

#[cfg(test)]
mod tests_symbols {
    use super::*;

    #[test]
    fn test_combinations_count() {
        let charset_empty: Box<[char]> = Box::from([]);
        let charset_one: Box<[char]> = Box::from(['a']);

        assert_eq!(combination_count(&charset_empty, 0, 0), 1, "0 symbols.");

        assert_eq!(combination_count(&charset_one, 0, 1), 2, "1 combination.");
    }

    #[test]
    #[should_panic]
    fn test_combinations_count_panic() {
        let charset: Box<[char]> = Box::from(['a']);
        assert_eq!(
            combination_count(&charset, 1, 0),
            0,
            "min length must be <= max length."
        )
    }
}
