#![allow(non_snake_case)]

use std::{sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Once,
}, thread::JoinHandle, time::Instant};
use std::thread;

use error_stack::IntoReport;
use jni::{
    objects::{JString, JValue},
    InitArgsBuilder, JavaVM, Executor, errors::Error,
};
use log::{info, debug};

use std::io::Write;
use chrono::Local;
use log::LevelFilter;

pub mod jni_wrap;

use crate::jni_wrap::{
    indices::{indices_create, indices_increment_by, indices_to_string},
    internal::InternalCrackData,
    CrackParam,
    result::CrackResult
};
use crate::jni_wrap::symbols::Builder;

fn main() {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter_level(LevelFilter::Info)
        .filter_level(LevelFilter::Debug)
        // .filter_level(LevelFilter::Trace)
        .init();

    let instant = Instant::now();

    let charset = Builder::new().with_latin_lc().build();

    let params = Arc::from(InternalCrackData::from(CrackParam::new(
        charset, 0, 4, "correct,",
    )));

    debug!("threads: {}.", params.thread_count());
    debug!("total: {}.", params.total_combos());
    debug!("per thread: {}.", params.combos_per_thread());

    let done = Arc::from(AtomicBool::from(false));

    // // Initialize JVM
    // let jvm = Arc::from(JavaVM::new(jvm_args).unwrap());
    // // Executor to handle memory allocation and Java Garbage Collection
    // let exec = Executor::new(jvm().clone());

    let mut handles = vec![];

    
    for tid in 0..params.thread_count() {
    // for tid in 0..1 {
        let mut indices = indices_create(
            params.crack_param().max_length(),
            params.crack_param().min_length(),
        );

        indices_increment_by(params.crack_param().charset(), &mut indices, tid)
            .expect("Increment failed");

        // debug!("{:#?}", indices);

        handles.push(task(/*exec.clone(), */params.clone(), done.clone(), indices, tid));
    }

    // wait for all threads
    let solution = handles
        .into_iter()
        .enumerate()
        .map(|(idx, h)| {
            println!("joining {}...", idx);
            h.join().unwrap()
        }) // result of the Option<String> from the threads
        .last(); // extract from the collection

    let seconds = instant.elapsed().as_secs_f64();

    let param =
        Arc::try_unwrap(params).unwrap_or_else(|_| panic!("There should only be one reference!"));
    let res = if let Some(Some(solution)) = solution {
        CrackResult::new_success(param, seconds, solution)
    } else {
        CrackResult::new_failure(param, seconds)
    };

    if let Some(solution) = res.solution() {
        println!("Password is: {}", solution);
        println!("Took {:.3}s.", res.duration_in_seconds());
    }
}

fn task(
    // exec: Executor,
    params: Arc<InternalCrackData>,
    done: Arc<AtomicBool>,
    mut indices: Box<[isize]>,
    tid: usize,
) -> JoinHandle<Option<String>>{
    // Counter for total iterations/total checked values
    let mut iteration_count = 0;

    let builder = thread::Builder::new().name(tid.to_string());

    builder.spawn(move || {
        let exec = Executor::new(jvm().clone());
        // let exec = Executor::new(jvm());
        exec.with_attached::<_, Option<String>, Error>(|jni_env| {
            // Prepare to call PasswordWrapper.PasswordWrapper() constructor
            let class_PasswordWrapper_name = "PasswordWrapper";
            let class_PasswordWrapper_JClass = jni_env
                .find_class(class_PasswordWrapper_name)
                .into_report()
                .unwrap();
            let class_PasswordWrapper_sig = "()V";

            // Load java PasswordWrapper class at classpath jvm argument option
            let class_PasswordWrapper_instance = jni_env
                .new_object(
                    &class_PasswordWrapper_JClass,
                    class_PasswordWrapper_sig,
                    &[],
                )
                .into_report()
                .unwrap();

            // // Prepare to get Password.hashCode() -> Int
            // let hashCode_name = "getHashCode";
            // let hashCode_sig = "()I";

            // // Call PasswordWrapper.hashCode() -> Int
            // let hashCode = jni_env
            //     .call_method(
            //         &class_PasswordWrapper_instance,
            //         hashCode_name,
            //         hashCode_sig,
            //         &[],
            //     )
            //     // .into_report()
            //     .unwrap();

            // println!("{} hash {:#?}.", tid, hashCode.i().into_report().unwrap());

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

            // debug!("start loop on {}", tid);

            loop {
                // tell about progress + stop if another thread found a solution
                {
                    if interrupt_count == 0 {
                        interrupt_count = INTERRUPT_COUNT_THRESHOLD;
                        if done.load(Ordering::SeqCst) {
                            info!("Thread {:>2} stops at {:>6.2}% progress because another thread found a solution", tid, get_percent(&params, iteration_count));
                            // unsafe {
                            //     jvm().detach_current_thread();
                            // }
                            break;
                        } else {
                            info!(
                                "Thread {:>2} is at {:>6.2}% progress",
                                tid,
                                get_percent(&params, iteration_count)
                            );
                        }
                    }
                    interrupt_count -= 1;
                }

                // debug!("begin cracking on {}.", tid);

                // the actual cracking
                {
                    let res = indices_increment_by(
                        params.crack_param().charset(),
                        &mut indices,
                        1,
                    );
                    if res.is_err() {
                        info!(
                            "Thread {:>2} checked all possible values without finding a solution. Done.",
                            tid
                        );
                        break;
                    }

                    // debug!("incremented on {}.", tid);

                    iteration_count += 1;

                    // build string
                    indices_to_string(
                        &mut current_crack_string,
                        params.crack_param().charset(),
                        &indices,
                    );

                    debug!("{} on {}.", current_crack_string, tid);

                    current_crack_string.push('\n');
                    let cracked = exec.with_attached::<_, bool, Error>(|jni_env| {
                        // debug!("begin jni calls on {}", tid);
                        // Prepare to call PasswordWrapper.writePipe(byte[] bytes, int offset, int length) -> String
                        let pipeWrite_name = "writePipe";
                        let pipeWrite_sig = "([BIII)Ljava/lang/String;";
                        // Prepare arg bytes
                        let pipeWrite_arg_bytes_string = current_crack_string.as_bytes();
                        let pipeWrite_arg_bytes_jarr = 
                        jni_env
                            .byte_array_from_slice(pipeWrite_arg_bytes_string)
                            .into_report()
                            .unwrap();

                        let pipeWrite_arg_bytes = JValue::from(&pipeWrite_arg_bytes_jarr);

                        // Prepare arg offset
                        let pipeWrite_arg_offset = JValue::from(0);

                        // Prepare arg length
                        let pipeWrite_arg_len = JValue::from(pipeWrite_arg_bytes_string.len() as i32);

                        // Prepare args
                        let pipeWrite_args = &[pipeWrite_arg_bytes, pipeWrite_arg_offset, pipeWrite_arg_len, JValue::from(tid as i32)];

                        // debug!("prepare to call write on {}", tid);

                        // Call PasswordWrapper.writePipe(byte[] bytes, int offset, int length) -> String
                        let response = jni_env
                            .call_method(
                                &class_PasswordWrapper_instance,
                                pipeWrite_name,
                                pipeWrite_sig,
                                pipeWrite_args,
                            )
                            .into_report()
                            .unwrap();

                        // debug!("wrote {}", tid);

                        let response_string: String = jni_env
                            .get_string(
                            &JString::from(response.l().into_report().unwrap()))
                            .unwrap()
                            .into();

                        current_crack_string.pop();

                        // debug!("from thread {}.", tid);
                        // debug!("response: {}", response_string.trim());
                        // debug!("current string: {}", current_crack_string);
                        // debug!("matches? {}", response_string.to_lowercase().contains(&params.crack_param().output_contains().to_lowercase()));

                        Ok(response_string.to_lowercase().contains(&params.crack_param().output_contains().to_lowercase()))
                    }).into_report().unwrap();

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
                    // else {
                    //     debug!("{} failed.", current_crack_string);
                    // }
                }
            }

            // Prepare to call Password.closePipe() -> V
            let closePipe_name = "closePipe";
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
            Ok(result)
        }).into_report().unwrap()
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

fn jvm() -> &'static Arc<JavaVM> {
// fn jvm() -> Arc<JavaVM> {
    static mut JVM: Option<Arc<JavaVM>> = None;
    static INIT: Once = Once::new();

    // let mut JVM: Option<Arc<JavaVM>> = None;

    INIT.call_once(|| {
        // JVM arguments
        let jvm_args = InitArgsBuilder::new()
            .version(jni::JNIVersion::V8)
            .option("-Xcheck:jni")
            .option("-Djava.class.path=/home/earth/rust/DN")
            // .option("-Djava.class_pw.path=/home/earth/password")
            .build()
            .into_report()
            .unwrap_or_else(|e| panic!("{:#?}", e));

        let jvm = JavaVM::new(jvm_args).unwrap_or_else(|e| panic!("{:#?}", e));

        unsafe {
            JVM = Some(Arc::new(jvm));
        }
    });

    unsafe { JVM.as_ref().unwrap() }
    // JVM.unwrap()
}