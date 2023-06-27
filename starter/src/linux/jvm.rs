use std::{
    error::Error,
    ffi::{c_void, CString},
    mem,
    path::{Path, PathBuf},
    ptr::null_mut,
};

use cstr::cstr;
use jni_sys::{jint, JNIEnv, JavaVM, JavaVMInitArgs, JavaVMOption, JNI_FALSE, JNI_OK, JNI_VERSION_1_8};

use libc::{dlerror, dlopen, dlsym, RTLD_NOW};

pub struct JavaRuntime {
    jvm_handle: *mut c_void,

    pub vm: *mut JavaVM,
    pub env: *mut JNIEnv,
}

impl Drop for JavaRuntime {
    fn drop(&mut self) {
        unsafe {
            let destroy_jvm = dlsym(self.jvm_handle, cstr!("DestroyJavaVM").as_ptr());
            if destroy_jvm != null_mut() {
                let destroy_jvm = mem::transmute::<_, unsafe extern "C" fn(*mut JavaVM) -> jint>(destroy_jvm);

                destroy_jvm(self.vm);
            }
        }
    }
}

fn find_jvm_so_by_jre_path(jre_path: &Path) -> Option<PathBuf> {
    let jvm_so = jre_path.join("lib/server/libjvm.so");
    if jvm_so.exists() {
        return Some(jvm_so);
    }

    let jvm_so = jre_path.join("lib/libjvm.so");
    if jvm_so.exists() {
        return Some(jvm_so);
    }

    None
}

fn find_jvm_so(app_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(jvm_so) = find_jvm_so_by_jre_path(&app_dir.join("jre")) {
        return Ok(jvm_so);
    }

    if let Ok(path) = which::which("java") {
        if let Some(jre_path) = path.parent().and_then(|p| p.parent()) {
            if let Some(jvm_so) = find_jvm_so_by_jre_path(jre_path) {
                return Ok(jvm_so);
            }
        }
    }

    if let Ok(jre_path) = std::env::var("JAVA_HOME") {
        if let Some(jvm_so) = find_jvm_so_by_jre_path(&Path::new(&jre_path)) {
            return Ok(jvm_so);
        }
    }

    Err("JavaRuntime not found".into())
}

pub fn load_jvm(app_dir: &Path, args: &[&str]) -> Result<JavaRuntime, Box<dyn Error>> {
    let jvm_so = find_jvm_so(app_dir)?;

    std::env::set_var(
        "LD_LIBRARY_PATH",
        jvm_so.parent().unwrap().to_str().unwrap().to_owned() + ";" + &std::env::var("LD_LIBRARY_PATH").unwrap_or("".to_owned()),
    );

    let jvm_so = CString::new(jvm_so.to_string_lossy().to_string())?;

    unsafe {
        let jvm_handle = dlopen(jvm_so.as_ptr(), RTLD_NOW);
        if jvm_handle == null_mut() {
            return Err(CString::from_raw(dlerror()).to_str()?.to_string().into());
        }

        let create_vm = dlsym(jvm_handle, cstr!("JNI_CreateJavaVM").as_ptr());
        if create_vm == null_mut() {
            return Err(CString::from_raw(dlerror()).to_str()?.to_string().into());
        }

        let create_vm =
            mem::transmute::<_, unsafe extern "C" fn(*mut *mut JavaVM, *mut *mut c_void, *mut c_void) -> jint>(create_vm);

        let args = args.iter().map(|s| CString::new(*s).unwrap()).collect::<Vec<_>>();
        let options = args
            .iter()
            .map(|p| JavaVMOption {
                optionString: p.as_ptr().cast_mut(),
                extraInfo: null_mut(),
            })
            .collect::<Vec<_>>();
        let mut vm_init_args = JavaVMInitArgs {
            version: JNI_VERSION_1_8,
            nOptions: options.len() as jint,
            options: options.as_ptr().cast_mut(),
            ignoreUnrecognized: JNI_FALSE,
        };

        let mut vm: *mut JavaVM = null_mut();
        let mut env: *mut JNIEnv = null_mut();

        let ret = create_vm(
            &mut vm,
            (&mut env as *mut *mut JNIEnv).cast(),
            (&mut vm_init_args as *mut JavaVMInitArgs).cast(),
        );
        if ret != JNI_OK {
            return Err("Create jvm failed".into());
        }

        Ok(JavaRuntime { jvm_handle, vm, env })
    }
}
