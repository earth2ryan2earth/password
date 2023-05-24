use crate::jni_wrap::param_interface::CrackParam;
use crate::jni_wrap::symbols::combination_count;

#[derive(Debug)]
pub(crate) struct InternalCrackData {
    crack_param: CrackParam,
    thread_count: usize,
    total_combos: usize,
    combos_per_thread: usize,
}

impl InternalCrackData {
    pub fn crack_param(&self) -> &CrackParam {
        &self.crack_param
    }

    pub fn thread_count(&self) -> usize {
        self.thread_count
    }

    pub fn total_combos(&self) -> usize {
        self.total_combos
    }

    pub fn combos_per_thread(&self) -> usize {
        self.combos_per_thread
    }
}

impl From<CrackParam> for InternalCrackData {
    fn from(cp: CrackParam) -> Self {
        let total_combos = combination_count(cp.charset(), cp.min_length(), cp.max_length());
        let mut thread_count = get_thread_count();
        // Assuming that the user will never have thousands of CPUs
        // there are so few possible permutations, that threading is unnecessary
        if thread_count > total_combos {
            thread_count = 1;
        }

        let combos_per_thread = total_combos / thread_count;
        Self {
            crack_param: cp,
            thread_count,
            total_combos,
            combos_per_thread,
        }
    }
}

fn get_thread_count() -> usize {
    let cpus = num_cpus::get();
    if cpus > 1 {
        cpus - 1
    } else {
        cpus
    }
}
