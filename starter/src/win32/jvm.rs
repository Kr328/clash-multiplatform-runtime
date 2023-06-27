use std::{
    error::Error,
    io,
    iter::once,
    mem,
    os::raw::c_void,
    path::{Path, PathBuf},
    ptr::null_mut,
};

use cstr::cstr;
use jni_sys::{jint, JNIEnv, JavaVM, JavaVMInitArgs, JavaVMOption, JNI_OK, JNI_VERSION_1_8};
use windows_sys::Win32::{
    Foundation::{HANDLE, HMODULE},
    System::LibraryLoader::{GetProcAddress, LoadLibraryExW},
};

use crate::{utils::strings::PathExt, win32::strings::Win32Strings};

fn find_jvm_dll_from_jre_path(jre_path: &Path) -> Option<PathBuf> {
    let jvm_dll_path = jre_path.join("bin\\server\\jvm.dll");
    if jvm_dll_path.exists() {
        return Some(jvm_dll_path);
    }

    let jvm_dll_path = jre_path.join("bin\\jvm.dll");
    if jvm_dll_path.exists() {
        return Some(jvm_dll_path);
    }

    None
}

fn find_jvm_dll(app_root: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(jvm_dll_path) = find_jvm_dll_from_jre_path(&app_root.join("jre")) {
        return Ok(jvm_dll_path);
    }

    if let Ok(java_bin_path) = which::which("java") {
        if let Some(jvm_dll_path) = java_bin_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| find_jvm_dll_from_jre_path(p))
        {
            return Ok(jvm_dll_path);
        }
    }

    if let Ok(jre_path) = std::env::var("JAVA_HOME") {
        if let Some(jvm_dll_path) = find_jvm_dll_from_jre_path(&PathBuf::from(jre_path)) {
            return Ok(jvm_dll_path);
        }
    }

    Err("JavaRuntime not found".into())
}

fn build_jvm_init_args(args: &[&str]) -> (Vec<Vec<u8>>, Vec<JavaVMOption>, JavaVMInitArgs) {
    let args = args
        .iter()
        .map(|s| (*s).chars().map(|c| c as u8).chain(once(0)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let options = args
        .iter()
        .map(|a| JavaVMOption {
            optionString: a.as_ptr().cast_mut().cast(),
            extraInfo: null_mut(),
        })
        .collect::<Vec<_>>();
    let vm_args = JavaVMInitArgs {
        version: JNI_VERSION_1_8,
        nOptions: options.len() as jint,
        options: options.as_ptr().cast_mut().cast(),
        ignoreUnrecognized: 0,
    };

    (args, options, vm_args)
}

pub struct JavaRuntime {
    jvm_module: HMODULE,

    pub vm: *mut JavaVM,
    pub env: *mut JNIEnv,
}

impl Drop for JavaRuntime {
    fn drop(&mut self) {
        unsafe {
            if let Some(destroy_func) = GetProcAddress(self.jvm_module, cstr!("DestroyJavaVM").as_ptr().cast()) {
                let destroy_func = mem::transmute::<_, unsafe extern "system" fn(*mut JavaVM) -> jint>(destroy_func);

                destroy_func(self.vm);
            }
        }
    }
}

pub fn load_jvm(app_root: &Path, args: &[&str]) -> Result<JavaRuntime, Box<dyn Error>> {
    let jvm_dll_path = find_jvm_dll(app_root)?.to_string_without_extend_length_mark();

    let jvm_module = unsafe {
        let jvm_dll_path = jvm_dll_path.to_win32_utf16();

        LoadLibraryExW(jvm_dll_path.as_ptr(), HANDLE::default(), 0)
    };
    if jvm_module == 0 {
        return Err(Box::new(io::Error::last_os_error()));
    }

    let create_jvm_func = unsafe {
        match GetProcAddress(jvm_module, cstr!("JNI_CreateJavaVM").as_ptr().cast()) {
            None => {
                return Err(format!("Invalid JavaRuntime: {}", jvm_dll_path).into());
            }
            Some(func) => {
                mem::transmute::<_, unsafe extern "system" fn(*mut *mut JavaVM, *mut *mut c_void, *mut c_void) -> jint>(func)
            }
        }
    };

    let (args, options, mut vm_args) = build_jvm_init_args(args);

    let mut vm: *mut JavaVM = null_mut();
    let mut env: *mut JNIEnv = null_mut();
    unsafe {
        if create_jvm_func(
            &mut vm,
            (&mut env as *mut *mut JNIEnv).cast(),
            (&mut vm_args as *mut JavaVMInitArgs).cast(),
        ) != JNI_OK
        {
            return Err("JNI_CreateJavaVM call failed".into());
        }
    }

    drop(args);
    drop(options);

    Ok(JavaRuntime { jvm_module, vm, env })
}
