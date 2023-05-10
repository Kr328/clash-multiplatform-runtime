#![windows_subsystem = "windows"]

use std::{error::Error, io::Write, path::Path, ptr::null_mut};

use clap::Parser;
use cstr::cstr;
use jni_sys::JNI_TRUE;

use crate::{
    dirs::current_app_dir,
    metadata::resolve_app_metadata,
    options::Options,
    startup::StartupParameters,
    utils::{errors::ErrorExt, java::jcall, strings::PathExt},
};

#[cfg(windows)]
mod win32;

#[cfg(target_os = "linux")]
mod linux;

mod dirs;
mod metadata;
mod options;
mod startup;
mod utils;

const APP_JAR_NAME: &str = "clash-multiplatform.jar";

fn run_app(options: &Options) -> Result<(), Box<dyn Error>> {
    let app_dir = current_app_dir().map_err(|e| e.with_message("App dir not found"))?;
    let classes_jar = app_dir.join(APP_JAR_NAME);
    let metadata = resolve_app_metadata(&classes_jar).map_err(|e| e.with_message("Resolve app metadata"))?;
    let parameters = StartupParameters::new(options, &metadata).map_err(|e| e.with_message("Resolve startup parameters"))?;

    #[cfg(windows)]
    win32::redirect::redirect_standard_output_to_file(&Path::new(&parameters.base_directory).join("app.log")).ok();

    #[cfg(target_os = "linux")]
    linux::redirect::redirect_standard_output_to_file(&Path::new(&parameters.base_directory).join("app.log")).ok();

    let classpath_opt = format!("-Djava.class.path={}", classes_jar.to_string_without_extend_length_mark());

    #[cfg(windows)]
    let runtime = win32::jvm::load_jvm(&app_dir, &[classpath_opt.as_str()]).map_err(|e| e.with_message("Load JavaRuntime"))?;

    #[cfg(target_os = "linux")]
    let runtime = linux::jvm::load_jvm(&app_dir, &[classpath_opt.as_str()]).map_err(|e| e.with_message("Load JavaRuntime"))?;

    let c_main = jcall!(runtime.env, FindClass, cstr!("com/github/kr328/clash/MainKt").as_ptr());
    if c_main == null_mut() {
        jcall!(runtime.env, ExceptionDescribe);

        return Err("Invalid application package".into());
    }

    let m_main = jcall!(
        runtime.env,
        GetStaticMethodID,
        c_main,
        cstr!("main").as_ptr(),
        cstr!("(Lcom/github/kr328/clash/StartupParameters;)V").as_ptr()
    );
    if m_main == null_mut() {
        jcall!(runtime.env, ExceptionDescribe);

        return Err("Invalid application package".into());
    }

    jcall!(
        runtime.env,
        CallStaticVoidMethod,
        c_main,
        m_main,
        parameters.new_java_object(runtime.env)
    );
    if jcall!(runtime.env, ExceptionCheck) == JNI_TRUE {
        jcall!(runtime.env, ExceptionDescribe);

        return Err("Unexpected exception".into());
    }

    Ok(())
}

fn main() {
    let options = Options::parse();

    if let Err(err) = run_app(&options) {
        #[cfg(windows)]
        win32::ui::show_error_message(&err.to_string());

        #[cfg(target_os = "linux")]
        std::io::stderr().write_all(err.to_string().as_bytes()).ok();
    }
}
