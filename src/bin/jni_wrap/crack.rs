use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

use crate::jni_wrap::{
    internal::InternalCrackData, param_interface::CrackParam, result::CrackResult, tasks::tasks,
};

pub fn crack(param: CrackParam) -> CrackResult {
    let param = InternalCrackData::from(param);
    let param = Arc::from(param);

    // shared atomic bool so that all threads can look if one already found a solution
    // so they can stop their work. This only gets checked at every millionth iteration
    // for better performance.
    let done = Arc::from(AtomicBool::from(false));
    let instant = Instant::now();
    let handles = tasks(param.clone(), done);

    // wait for all threads
    let solution = handles
        .into_iter()
        .flat_map(|h| h.join().unwrap()) // result of the Option<String> from the threads
        .last(); // extract from the collection

    let seconds = instant.elapsed().as_secs_f64();

    let param =
        Arc::try_unwrap(param).unwrap_or_else(|_| panic!("There should only be one reference!"));
    if let Some(solution) = solution {
        CrackResult::new_success(param, seconds, solution)
    } else {
        CrackResult::new_failure(param, seconds)
    }
}
