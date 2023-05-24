pub mod jni_wrap;
use jni_wrap::{crack, symbols::Builder, CrackParam};

fn main() {
    let charset = Builder::new().with_latin_lc().build();

    let res = crack(CrackParam::new(charset, 0, 1, "correct!"));

    if let Some(solution) = res.solution() {
        println!("Password is: {}", solution);
        println!("Took {:.3}s", res.duration_in_seconds());
    }
}
