use std::{
    ffi::OsStr,
    // io::BufReader,
    process::{ChildStdin, ChildStdout, Command, Stdio},
};

use crate::parameters::Spawn;

/// Attempts to spawn a child with the given script & args fields of the spawn struct.
/// Returns false if program fails to spawn.
/// Returns false if program outputs to stderr.
/// Returns true if neither.
pub fn does_child_err<
    // I must implement IntoIterator with Item of type S & Clone
    I: IntoIterator<Item = S> + std::clone::Clone,
    // S must implement AsRef to type OsStr & Clone
    S: AsRef<OsStr> + std::clone::Clone,
>(
    spawn: Spawn<I, S>,
) -> bool {
    let mut child = Command::new(spawn.script())
        .args(spawn.args().clone())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Child failed to spawn.");

    match child.try_wait() {
        Ok(None) => false,
        Err(_) => false,
        Ok(Some(_)) => {
            // panic!(
            // "{}",
            // std::str::from_utf8(&child.wait_with_output().unwrap().stderr.into_boxed_slice())
            //     .unwrap()
            // )
            panic!("Program errored.")
        }
    }
}

/// Spawns a child with the given script & args fields of the spawn struct.
/// Returns child's stdin & stdout.
pub fn child_spawn<
    I: IntoIterator<Item = S> + std::clone::Clone,
    S: AsRef<OsStr> + std::clone::Clone,
>(
    spawn: Spawn<I, S>,
) -> (ChildStdin, ChildStdout) {
    let mut child = Command::new(spawn.script())
        .args(spawn.args().clone())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .spawn()
        .expect("Child failed to spawn");
    (child.stdin.take().unwrap(), child.stdout.take().unwrap())
}
