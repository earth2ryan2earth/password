use std::{error::Error, thread, vec};

use error_stack::IntoReport;
use jni::{
    objects::{JObject, JValueGen},
    InitArgsBuilder, JavaVM,
};

fn main() {
    let mut handles = vec![];

    for tid in 0..10 {
        let builder = thread::Builder::new().name(tid.to_string());
        let handle = builder
            .spawn(move || {
                let jvm_args = InitArgsBuilder::new()
                    .version(jni::JNIVersion::V8)
                    .option("-Xcheck:jni")
                    .option("-Djava.class.path=/home/earth/rust/DN")
                    .build()
                    .into_report()
                    .unwrap();

                let jvm = JavaVM::new(jvm_args)
                    // .into_report()
                    // .unwrap();
                    .unwrap_or_else(|e| panic!("{:#?}", e.source()));
                let mut jni_env = jvm.attach_current_thread().into_report().unwrap();

                let class_name = "PasswordWrapper";
                let class = jni_env.find_class(class_name).into_report().unwrap();

                let method_name = "main";
                let method_signature = "([Ljava/lang/String;)V";

                let jobject = JObject::from(jni_env.new_string("").unwrap());

                let args = &[JValueGen::from(&jobject)];

                jni_env
                    .call_static_method(&class, method_name, method_signature, args)
                    .into_report()
                    .unwrap();

                println!("Iter {}.", tid);
            })
            .unwrap();
        handles.push(handle);
    }

    handles.into_iter().for_each(|h| h.join().unwrap());
}
