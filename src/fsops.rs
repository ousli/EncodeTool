use chrono::{DateTime, Local};
use filetime::FileTime;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Normalises a pasted / dragged path: surrounding quotes, escaped spaces and
/// stray (non-breaking) whitespace added by the terminal or Finder.
pub fn clean_path(raw: &str) -> String {
    let is_space = |c: char| c.is_whitespace() || c == '\u{a0}';
    let mut s = raw.trim_matches(is_space);
    for q in ['"', '\''] {
        if s.len() >= 2 && s.starts_with(q) && s.ends_with(q) {
            s = &s[1..s.len() - 1];
        }
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek().is_some_and(|n| n.is_whitespace() || "()'\"&[]".contains(*n)) {
            continue;
        }
        out.push(c);
    }
    let out = out.trim_matches(is_space).to_string();
    match out.strip_prefix('~') {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("{}{rest}", home()),
        _ => out,
    }
}

pub fn home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/".into())
}

pub fn is_video(path: &Path) -> bool {
    let hidden = path.file_name().and_then(|n| n.to_str()).is_none_or(|n| n.starts_with('.'));
    let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase());
    !hidden && matches!(ext.as_deref(), Some("mp4" | "mov"))
}

/// Videos (mp4/mov) directly inside `dir`, sorted by name.
pub fn list_videos(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.path())
        .filter(|p| is_video(p))
        .collect();
    files.sort();
    Ok(files)
}

/// `YYYY-MM-DD_HHMM` from the file's modification time (local time).
pub fn date_prefix(path: &Path) -> std::io::Result<String> {
    let modified = fs::metadata(path)?.modified()?;
    Ok(DateTime::<Local>::from(modified).format("%Y-%m-%d_%H%M").to_string())
}

/// Prefixes the file name with its date and returns the new name; files
/// already prefixed are left alone (None).
pub fn rename_with_date(path: &Path) -> Result<Option<String>, String> {
    let prefix = date_prefix(path).map_err(|e| e.to_string())?;
    let name = path.file_name().and_then(|n| n.to_str()).ok_or("nom de fichier invalide")?;
    if name.starts_with(&format!("{prefix}_")) {
        return Ok(None);
    }
    let new_name = format!("{prefix}_{name}");
    let target = path.with_file_name(&new_name);
    if target.exists() {
        return Err(format!("{new_name} existe déjà"));
    }
    fs::rename(path, &target).map_err(|e| e.to_string())?;
    Ok(Some(new_name))
}

/// Copies creation and modification dates from `src` to `dest`. On macOS,
/// moving the mtime before the birth time lowers the birth time, so the mtime
/// is set twice (creation first, then modification).
pub fn copy_dates(src: &Path, dest: &Path) -> std::io::Result<()> {
    let meta = fs::metadata(src)?;
    let modified = FileTime::from_last_modification_time(&meta);
    let created = meta.created().map(FileTime::from).unwrap_or(modified);
    filetime::set_file_mtime(dest, created)?;
    filetime::set_file_mtime(dest, modified)
}

#[derive(Serialize)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct Browse {
    pub path: String,
    pub parent: Option<String>,
    pub dirs: Vec<DirEntry>,
    pub files: Vec<DirEntry>,
    pub video_count: usize,
    pub shortcuts: Vec<DirEntry>,
}

pub fn browse(raw: &str, show_hidden: bool, ext: Option<&str>) -> Result<Browse, String> {
    let cleaned = clean_path(raw);
    let dir = if cleaned.is_empty() { PathBuf::from(home()) } else { PathBuf::from(cleaned) };
    if !dir.is_dir() {
        return Err(format!("Dossier introuvable : {}", dir.display()));
    }
    let mut dirs: Vec<DirEntry> = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            (show_hidden || !name.starts_with('.')).then(|| DirEntry { path: e.path().to_string_lossy().into_owned(), name })
        })
        .collect();
    dirs.sort_by_key(|d| d.name.to_lowercase());
    let mut files: Vec<DirEntry> = match ext {
        Some(ext) => fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case(ext)))
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                (show_hidden || !name.starts_with('.')).then(|| DirEntry { path: e.path().to_string_lossy().into_owned(), name })
            })
            .collect(),
        None => Vec::new(),
    };
    files.sort_by_key(|d| d.name.to_lowercase());
    let home = home();
    let shortcuts = [("Accueil", home.clone()), ("Bureau", format!("{home}/Desktop")), ("Films", format!("{home}/Movies")), ("Volumes", "/Volumes".into())]
        .into_iter()
        .filter(|(_, p)| Path::new(p).is_dir())
        .map(|(name, path)| DirEntry { name: name.into(), path })
        .collect();
    Ok(Browse {
        path: dir.to_string_lossy().into_owned(),
        parent: dir.parent().map(|p| p.to_string_lossy().into_owned()),
        video_count: list_videos(&dir).map(|v| v.len()).unwrap_or(0),
        dirs,
        files,
        shortcuts,
    })
}

/// Native macOS folder dialog. Returns None when the user cancels.
pub async fn pick_folder(file: bool) -> Result<Option<String>, String> {
    let script = if file {
        "POSIX path of (choose file with prompt \"Choisir un fichier .cube\")"
    } else {
        "POSIX path of (choose folder with prompt \"Choisir un dossier\")"
    };
    let out = Command::new("osascript")
        .args(["-e", script])
        .output()
        .await
        .map_err(|e| format!("osascript indisponible : {e}"))?;
    if !out.status.success() {
        return Ok(None);
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok((!path.is_empty()).then(|| path.trim_end_matches('/').to_string()).filter(|p| !p.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_dropped_paths() {
        assert_eq!(clean_path("  '/Users/me/My\\ Videos'\u{a0}"), "/Users/me/My Videos");
        assert_eq!(clean_path("\"/a/b\""), "/a/b");
    }

    #[test]
    fn detects_videos_case_insensitively() {
        assert!(is_video(Path::new("/x/CLIP.MP4")));
        assert!(is_video(Path::new("/x/a.mov")));
        assert!(!is_video(Path::new("/x/._a.mov")));
        assert!(!is_video(Path::new("/x/a.mkv")));
    }

    #[test]
    fn renames_once() {
        let dir = std::env::temp_dir().join(format!("encodetool-rename-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("clip.mp4");
        fs::write(&file, b"x").unwrap();
        let new_name = rename_with_date(&file).unwrap().expect("renamed");
        assert!(new_name.ends_with("_clip.mp4"));
        let renamed = dir.join(&new_name);
        assert!(rename_with_date(&renamed).unwrap().is_none());
        fs::remove_dir_all(&dir).unwrap();
    }
}
