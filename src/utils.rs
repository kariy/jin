use std::{fs, path::Path, str::FromStr};

use async_std::path::PathBuf;
use color_eyre::{eyre, Result};
use serde::Serialize;

pub fn dump_to_file<P, C>(path: Option<P>, contents: C) -> Result<()>
where
    P: AsRef<Path>,
    C: Serialize,
{
    if let Some(path) = path {
        let mut path = path.as_ref().to_path_buf();

        let p = path.clone();
        let filename = p
            .file_name()
            .ok_or(eyre::eyre!("output target must be a file"))?;

        path.pop();
        fs::create_dir_all(&path)?;
        path.push(filename);
        fs::write(path, serde_json::to_string_pretty(&contents)?)?;
    } else {
        let mut path = PathBuf::from_str("./output")?;
        fs::create_dir_all(&path)?;
        path.push("storage_slot.json");
        fs::write(path, serde_json::to_string_pretty(&contents)?)?;
    }

    Ok(())
}
