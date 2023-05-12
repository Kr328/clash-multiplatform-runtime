use std::{env::current_exe, error::Error, ptr::null_mut};

use cstr::cstr;
use jni_sys::{jint, jobject, jsize, JNIEnv, JNI_FALSE, JNI_TRUE};

use crate::{
    dirs::default_base_dir,
    metadata::Metadata,
    options::Options,
    utils::{
        java::{jcall, JStringExt},
        strings::PathExt,
    },
};

pub struct StartupParameters {
    pub base_directory: String,
    pub no_shortcut: bool,
    pub hide_window: bool,
    pub starter: String,
    pub starter_arguments: Vec<String>,
}

impl StartupParameters {
    pub fn new(options: &Options, metadata: &Metadata) -> Result<Self, Box<dyn Error>> {
        let base_directory = if options.base_directory.is_empty() {
            default_base_dir(metadata)?.to_string_without_extend_length_mark()
        } else {
            options.base_directory.to_owned()
        };
        let starter = current_exe()?.to_string_without_extend_length_mark();
        let starter_arguments = std::env::args().skip(1).collect::<Vec<_>>();

        return Ok(StartupParameters {
            base_directory,
            no_shortcut: options.no_shortcut,
            hide_window: options.hide_window,
            starter,
            starter_arguments,
        });
    }

    pub fn new_java_object(&self, env: *mut JNIEnv) -> jobject {
        let base_directory = self.base_directory.to_java_string(env);
        let no_shortcut = if self.no_shortcut { JNI_TRUE } else { JNI_FALSE };
        let hide_window = if self.hide_window { JNI_TRUE } else { JNI_FALSE };
        let starter = self.starter.to_java_string(env);
        let c_string = jcall!(env, FindClass, cstr!("java/lang/String").as_ptr());
        let starter_arguments = jcall!(
            env,
            NewObjectArray,
            self.starter_arguments.len() as jsize,
            c_string,
            null_mut()
        );

        for (idx, value) in self.starter_arguments.iter().enumerate() {
            jcall!(
                env,
                SetObjectArrayElement,
                starter_arguments,
                idx as jsize,
                value.to_java_string(env)
            );
        }

        let class = jcall!(env, FindClass, cstr!("com/github/kr328/clash/StartupParameters").as_ptr());

        let constructor = jcall!(
            env,
            GetMethodID,
            class,
            cstr!("<init>").as_ptr(),
            cstr!("(Ljava/lang/String;ZZLjava/lang/String;[Ljava/lang/String;)V").as_ptr()
        );

        jcall!(
            env,
            NewObject,
            class,
            constructor,
            base_directory,
            no_shortcut as jint,
            hide_window as jint,
            starter,
            starter_arguments
        )
    }
}
