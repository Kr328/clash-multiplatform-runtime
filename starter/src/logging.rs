use std::{
    error::Error,
    fs::File,
    io,
    io::{BufRead, Write},
    path::Path,
};

const MAX_LOGFILE_LENGTH: usize = 20 * 1024 * 1024;

struct Discard {}

impl Write for Discard {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        return Ok(buf.len());
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn open_stdout_logfile_or_discard(base_dir: &Path) -> Box<dyn Write> {
    let new_file = base_dir.join("app.log");
    let old_file = base_dir.join("app.log.old");
    if new_file.exists() {
        let _ = std::fs::remove_file(&old_file);
        let _ = std::fs::rename(&new_file, &old_file);
    }

    if let Ok(file) = File::options().write(true).truncate(true).create(true).open(new_file) {
        Box::new(file)
    } else {
        Box::new(Discard {})
    }
}

pub fn redirect_stdout_to_logfile(base_dir: &Path) -> Result<(), Box<dyn Error>> {
    let base_dir = base_dir.to_path_buf();
    let (stdout_reader, stdout_writer) = os_pipe::pipe()?;

    #[cfg(target_os = "linux")]
    {
        use std::os::fd::IntoRawFd;

        crate::linux::redirect::set_standard_input_output(
            crate::linux::redirect::StandardOutputID::Output,
            stdout_writer.into_raw_fd(),
        )?;
    }

    #[cfg(windows)]
    {
        use std::os::windows::io::IntoRawHandle;

        crate::win32::redirect::set_standard_input_output(
            crate::win32::redirect::StandardInputOutput::Output,
            stdout_writer.into_raw_handle(),
        )?;
    }

    std::thread::spawn(move || {
        let mut reader = io::BufReader::new(stdout_reader);
        let mut writer: Box<dyn Write> = open_stdout_logfile_or_discard(&base_dir);

        let mut line = Vec::new();
        let mut written = 0usize;
        loop {
            line.clear();

            match reader.read_until(b'\n', &mut line) {
                Ok(size) => {
                    if size == 0 {
                        break;
                    }
                }
                Err(_) => break,
            }

            let reopen = match writer.write_all(&line) {
                Ok(_) => {
                    written += line.len();

                    written >= MAX_LOGFILE_LENGTH
                }
                Err(_) => true,
            };
            if reopen {
                writer = open_stdout_logfile_or_discard(&base_dir);
                written = 0;
            }
        }
    });

    return Ok(());
}

pub fn redirect_stderr_to_logfile(base_dir: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(base_dir.join("app_err.log"))?;

    #[cfg(target_os = "linux")]
    {
        use std::os::fd::IntoRawFd;

        crate::linux::redirect::set_standard_input_output(crate::linux::redirect::StandardOutputID::Error, file.into_raw_fd())
    }

    #[cfg(windows)]
    {
        use std::os::windows::io::IntoRawHandle;

        crate::win32::redirect::set_standard_input_output(
            crate::win32::redirect::StandardInputOutput::Error,
            file.into_raw_handle(),
        )
    }
}
