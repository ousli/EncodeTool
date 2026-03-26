use anyhow::Result;
use chrono::{DateTime, Local};
use std::fs;
use std::path::Path;
use filetime::{FileTime, set_file_times};

pub fn get_formatted_date(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)?;
    let modified: DateTime<Local> = metadata.modified()?.into();
    Ok(modified.format("%Y-%m-%d_%H%M").to_string())
}

pub fn apply_original_dates(src: &Path, dest: &Path) -> Result<()> {
    let metadata = fs::metadata(src)?;
    
    // On récupère atime et mtime. La date de création est plus complexe à modifier sur macOS en Rust pur,
    // donc on se concentre sur mtime (le plus important pour le classement).
    let mtime = FileTime::from_last_modification_time(&metadata);
    let atime = FileTime::from_last_access_time(&metadata);
    
    set_file_times(dest, atime, mtime)?;
    Ok(())
}

pub fn is_video(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext = ext.to_lowercase();
        ext == "mp4" || ext == "mov"
    } else {
        false
    }
}
