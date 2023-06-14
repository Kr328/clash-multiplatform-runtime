#![windows_subsystem = "windows"]

use std::{error::Error, fs::File, io::Write, path::Path, ptr::null_mut};

use clap::Parser;
use cstr::cstr;
use jni_sys::JNI_TRUE;

use crate::{
    dirs::current_app_dir,
    logging::redirect_pipe_logfile,
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
mod logging;
mod metadata;
mod options;
mod startup;
mod utils;

const APP_JAR_NAME: &str = "clash-multiplatform.jar";
const MAX_HEAP_USAGE_MB: usize = 512;

fn run_app(options: &Options) -> Result<(), Box<dyn Error>> {
    let app_dir = current_app_dir().map_err(|e| e.with_message("App dir not found"))?;
    let classes_jar = app_dir.join(APP_JAR_NAME);
    let metadata = resolve_app_metadata(&classes_jar).map_err(|e| e.with_message("Resolve app metadata"))?;
    let parameters = StartupParameters::new(options, &metadata).map_err(|e| e.with_message("Resolve startup parameters"))?;

    let (reader, writer) = os_pipe::pipe()?;
    let redirector = redirect_pipe_logfile(reader, &Path::new(&parameters.base_directory));

    #[cfg(windows)]
    {
        use std::os::windows::io::{AsRawHandle, IntoRawHandle};
        use win32::redirect::*;

        let _ = set_standard_input_output(
            StandardInputOutput::Input,
            open_null_device(File::options().read(true)).into_raw_handle(),
        );

        let _ = set_standard_input_output(StandardInputOutput::Output, writer.as_raw_handle());
        let _ = set_standard_input_output(StandardInputOutput::Error, writer.as_raw_handle());
    }

    #[cfg(target_os = "linux")]
    {
        use linux::redirect::*;
        use std::os::unix::io::{AsRawFd, IntoRawFd};

        let _ = set_standard_input_output(
            StandardInputOutput::Input,
            open_null_device(File::options().read(true)).into_raw_fd(),
        );

        let _ = set_standard_input_output(StandardInputOutput::Output, writer.as_raw_fd());
        let _ = set_standard_input_output(StandardInputOutput::Error, writer.as_raw_fd());
    }

    let classpath_opt = format!("-Djava.class.path={}", classes_jar.to_string_without_extend_length_mark());
    let max_heap_opt = format!("-Xmx{}m", MAX_HEAP_USAGE_MB);

    let init_opts: [&str; 3] = [&classpath_opt, &max_heap_opt, "-XX:+UseSerialGC"];

    #[cfg(windows)]
    let runtime = win32::jvm::load_jvm(&app_dir, &init_opts).map_err(|e| e.with_message("Load JavaRuntime"))?;

    #[cfg(target_os = "linux")]
    let runtime = linux::jvm::load_jvm(&app_dir, &init_opts).map_err(|e| e.with_message("Load JavaRuntime"))?;

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

    drop(writer);

    let _ = redirector.join();

    Ok(())
}

fn main() {
    let options = Options::parse();

    if let Err(err) = run_app(&options) {
        _ = std::io::stderr().write_fmt(format_args!("[Starter] err={} | Launch failed", err));

        #[cfg(windows)]
        win32::ui::show_error_message(&err.to_string());
    }
}
