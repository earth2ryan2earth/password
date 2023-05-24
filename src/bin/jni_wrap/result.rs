use crate::jni_wrap::internal::InternalCrackData;

pub struct CrackResult {
    solution: Option<String>,
    thread_count: usize,
    combinations_total: usize,
    combinations_per_thread: usize,
    duration_in_seconds: f64,
}

impl CrackResult {
    fn new(cp: InternalCrackData, duration_in_seconds: f64, solution: Option<String>) -> Self {
        Self {
            solution,
            thread_count: cp.thread_count(),
            combinations_total: cp.total_combos(),
            combinations_per_thread: cp.combos_per_thread(),
            duration_in_seconds,
        }
    }

    pub(crate) fn new_failure(cp: InternalCrackData, seconds_as_fraction: f64) -> Self {
        Self::new(cp, seconds_as_fraction, None)
    }

    pub(crate) fn new_success(
        cp: InternalCrackData,
        seconds_as_fraction: f64,
        solution: String,
    ) -> Self {
        Self::new(cp, seconds_as_fraction, Some(solution))
    }

    pub const fn is_failure(&self) -> bool {
        self.solution.is_none()
    }

    pub const fn is_success(&self) -> bool {
        self.solution.is_some()
    }

    pub const fn solution(&self) -> &Option<String> {
        &self.solution
    }

    pub const fn thread_count(&self) -> usize {
        self.thread_count
    }

    pub const fn combinations_total(&self) -> usize {
        self.combinations_total
    }

    pub const fn combinations_per_thread(&self) -> usize {
        self.combinations_per_thread
    }

    pub const fn duration_in_seconds(&self) -> f64 {
        self.duration_in_seconds
    }
}
