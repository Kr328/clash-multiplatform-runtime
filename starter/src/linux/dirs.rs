use std::{error::Error, path::PathBuf};

pub fn current_user_config_directory() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(dir) = home::home_dir() {
        Ok(dir.join(".config"))
    } else {
        Err("User home directory not found".into())
    }
}
