use crate::model::AppData;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn data_path() -> PathBuf {
    std::env::home_dir()
        .map(|h| h.join(".local/share/taskman/data.json"))
        .unwrap_or_else(|| PathBuf::from("data.json"))
}

fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn load() -> std::io::Result<AppData> {
    let path = data_path();
    if !path.exists() {
        return Ok(AppData::default());
    }
    let text = fs::read_to_string(&path)?;
    let data: AppData = serde_json::from_str(&text).unwrap_or_else(|_| AppData::default());
    Ok(data)
}

pub fn save(data: &AppData) -> std::io::Result<()> {
    let path = data_path();
    ensure_parent(&path)?;
    let text = serde_json::to_string_pretty(data)?;
    let mut file = fs::File::create(&path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}
