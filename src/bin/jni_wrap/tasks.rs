use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

use error_stack::IntoReport;
use jni::{
    objects::{JString, JValue},
    InitArgsBuilder, JavaVM,
};
use log::{info, trace};

use crate::jni_wrap::{
    indices::{indices_create, indices_increment_by, indices_to_string},
    internal::InternalCrackData,
};

pub(crate) fn tasks(
    params: Arc<InternalCrackData>,
    done: Arc<AtomicBool>,
) -> Vec<thread::JoinHandle<Option<String>>> {
    let mut handles = vec![];
    // spawn thread for each cpu
    for tid in 0..params.thread_count() {
        // indices object, that each thread gets as starting point
        let mut indices = indices_create(
            params.crack_param().max_length(),
            params.crack_param().min_length(),
        );

        // alternate indices object for the next thread
        indices_increment_by(params.crack_param().charset(), &mut indices, tid)
            .expect("Increment failed");

        handles.push(task(params.clone(), done.clone(), indices, tid));
    }
    handles
}

fn task(
    params: Arc<InternalCrackData>,
    done: Arc<AtomicBool>,
    mut indices: Box<[isize]>,
    tid: usize,
) -> thread::JoinHandle<Option<String>> {
    // Counter for total iterations/total checked values
    let mut iteration_count = 0;

    let builder = thread::Builder::new().name(tid.to_string());

    builder.spawn(move || {
        // JVM arguments
        let jvm_args = InitArgsBuilder::new()
            .version(jni::JNIVersion::V8)
            .option("-Xcheck:jni")
            .option("-Djava.class.path=/home/earth/rust/DN")
            // .option("-Djava.class_pw.path=/home/earth/password")
            .build()
            .into_report()
            .unwrap();

        // Initialize JVM
        let jvm = JavaVM::new(jvm_args).into_report().unwrap();
        // Attach JVM to current thread
        let mut jni_env = jvm.attach_current_thread().into_report().unwrap();

        // Prepare to call PasswordWrapper.PasswordWrapper() constructor
        #[allow(non_snake_case)]
        let class_PasswordWrapper_name = "PasswordWrapper";
        #[allow(non_snake_case)]
        let class_PasswordWrapper_JClass = jni_env
            .find_class(class_PasswordWrapper_name)
            .into_report()
            .unwrap();
        #[allow(non_snake_case)]
        let class_PasswordWrapper_sig = "()V";

        // Load java PasswordWrapper class at classpath jvm argument option
        #[allow(non_snake_case)]
        let class_PasswordWrapper_instance = jni_env
            .new_object(
                &class_PasswordWrapper_JClass,
                class_PasswordWrapper_sig,
                &[],
            )
            .into_report()
            .unwrap();

        // reserve a string buffer with the maximum needed size; in the worst case it can contain
        // indices.len() * 4 bytes, because UTF-8 chars can be at most 4 byte long. Because
        // I prevent the allocation for a string in every iteration and do this only once,
        // I cauld improve the performance even further.
        let mut current_crack_string = String::with_capacity(indices.len() * 4);

        // The result that the thread calculated/found
        let mut result = None;

        /// The amount of iterations after the thread checks if another thread
        /// is already done, so that we can stop further work. We do this only after
        /// a few millions iterations to keep the overhead low. Tests on my machine
        /// (i5-10600K) showed that 2 million iterations take about 1s - this should be okay
        /// because the overhead is not that big. A test already showed that
        /// increasing this has no real impact on the iterations per s.
        const INTERRUPT_COUNT_THRESHOLD: usize = 100;
        let mut interrupt_count = INTERRUPT_COUNT_THRESHOLD;

        loop {
            // tell about progress + stop if another thread found a solution
            {
                if interrupt_count == 0 {
                    interrupt_count = INTERRUPT_COUNT_THRESHOLD;
                    if done.load(Ordering::SeqCst) {
                        trace!("Thread {:>2} stops at {:>6.2}% progress because another thread found a solution", tid, get_percent(&params, iteration_count));
                        println!("Thread {:>2} stops at {:>6.2}% progress because another thread found a solution", tid, get_percent(&params, iteration_count));
                        break;
                    } else {
                        trace!(
                            "Thread {:>2} is at {:>6.2}% progress",
                            tid,
                            get_percent(&params, iteration_count)
                        );
                        println!(
                            "Thread {:>2} is at {:>6.2}% progress",
                            tid,
                            get_percent(&params, iteration_count)
                        );
                    }
                }
                interrupt_count -= 1;
            }

            // the actual cracking
            {
                let res = indices_increment_by(
                    params.crack_param().charset(),
                    &mut indices,
                    params.thread_count(),
                );
                if res.is_err() {
                    info!(
                        "Thread {:>2} checked all possible values without finding a solution. Done.",
                        tid
                    );
                    println!(
                        "Thread {:>2} checked all possible values without finding a solution. Done.",
                        tid
                    );
                    break;
                }

                iteration_count += 1;

                // build string
                indices_to_string(
                    &mut current_crack_string,
                    params.crack_param().charset(),
                    &indices,
                );

                current_crack_string.push('\n');

                // Prepare to call PasswordWrapper.writePipe(byte[] bytes, int offset, int length) -> V
                #[allow(non_snake_case)]
                let pipeWrite_name = "writePipe";
                #[allow(non_snake_case)]
                let pipeWrite_sig = "([BII)V";
                #[allow(non_snake_case)]
                // Prepare arg bytes
                let pipeWrite_arg_bytes_string = b"q\n";
                #[allow(non_snake_case)]
                let pipeWrite_arg_bytes_jarr = jni_env
                    .byte_array_from_slice(pipeWrite_arg_bytes_string)
                    .into_report()
                    .unwrap();
                #[allow(non_snake_case)]
                let pipeWrite_arg_bytes = JValue::from(&pipeWrite_arg_bytes_jarr);

                // Prepare arg offset
                #[allow(non_snake_case)]
                let pipeWrite_arg_offset = JValue::from(0);

                // Prepare arg length
                #[allow(non_snake_case)]
                let pipeWrite_arg_len = JValue::from(pipeWrite_arg_bytes_string.len() as i32);

                // Prepare args
                #[allow(non_snake_case)]
                let pipeWrite_args =
                    &[pipeWrite_arg_bytes, pipeWrite_arg_offset, pipeWrite_arg_len];

                // Call PasswordWrapper.writePipe(byte[] bytes, int offset, int length) -> V
                #[allow(non_snake_case)]
                jni_env
                    .call_method(
                        &class_PasswordWrapper_instance,
                        pipeWrite_name,
                        pipeWrite_sig,
                        pipeWrite_args,
                    )
                    .into_report()
                    .unwrap();

                // Prepare to call Password.spawn() -> String
                #[allow(non_snake_case)]
                let spawn_name = "spawn";
                #[allow(non_snake_case)]
                let spawn_sig = "()Ljava.lang.String;";

                // Call Password.spawn() -> String
                let response = jni_env
                    .call_method(&class_PasswordWrapper_instance, spawn_name, spawn_sig, &[])
                    .into_report()
                    .unwrap();

                let response_string: String = jni_env
                    .get_string(&JString::from(response.l().into_report().unwrap()))
                    .unwrap()
                    .into();

                let cracked = response_string.to_ascii_lowercase().contains(&params.crack_param().output_contains().to_ascii_lowercase());

                current_crack_string.pop();

                // transform; e.g. hashing
                // extra parentheses to prevent "field, not a method" error
                if cracked {
                    info!(
                        "Thread {:>2} found solution \"{}\" at a progress of {:>6.2}%!",
                        tid,
                        current_crack_string,
                        get_percent(&params, iteration_count)
                    );
                    // let other threads know we are done
                    done.store(true, Ordering::SeqCst);
                    result = Some(current_crack_string);
                    break;
                }
            }
        }

        // Prepare to call Password.closePipe() -> V
        #[allow(non_snake_case)]
        let closePipe_name = "closePipe";
        #[allow(non_snake_case)]
        let closePipe_sig = "()V";

        // Call Password.closePipe() -> V
        jni_env
            .call_method(
                &class_PasswordWrapper_instance,
                closePipe_name,
                closePipe_sig,
                &[],
            )
            .into_report()
            .unwrap();
        result
    }).into_report().unwrap()
}

/// Returns the percent of all possible iterations that the current thread has already
/// executed.
#[inline]
fn get_percent(cp: &InternalCrackData, iteration_count: u64) -> f32 {
    let total = cp.combos_per_thread() as f32;
    let current = iteration_count as f32;
    current / total * 100.0
}
