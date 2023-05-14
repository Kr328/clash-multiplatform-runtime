use std::{
    fs::File,
    io::{BufRead, Write},
    path::Path,
};

use os_pipe::PipeReader;

const MAX_LOGFILE_LENGTH: usize = 20 * 1024 * 1024;

struct Discard {}

impl Write for Discard {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        return Ok(buf.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn open_logfile_or_discard(base_dir: &Path) -> Box<dyn Write> {
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

pub fn redirect_pipe_logfile(pipe: PipeReader, base_dir: &Path) -> std::thread::JoinHandle<()> {
    let base_dir = base_dir.to_path_buf();

    return std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(pipe);
        let mut writer: Box<dyn Write> = open_logfile_or_discard(&base_dir);

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
                writer = open_logfile_or_discard(&base_dir);
                written = 0;
            }
        }
    });
}
