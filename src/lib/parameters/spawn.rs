use std::ffi::OsStr;

#[derive(Debug, Clone)]
pub struct Spawn<I, S> {
    script: S,
    args: I,
    look_for_output: S,
}

impl<I: IntoIterator<Item = S> + std::clone::Clone, S: AsRef<OsStr> + std::clone::Clone>
    Spawn<I, S>
{
    pub fn new(script: S, args: I, look_for_output: S) -> Self {
        Self {
            script,
            args,
            look_for_output,
        }
    }

    pub fn script(&self) -> &S {
        &self.script
    }
    pub fn args(&self) -> &I {
        &self.args
    }
    pub fn look_for_output(&self) -> &S {
        &self.look_for_output
    }
}
