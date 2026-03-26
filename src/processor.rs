use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::models::{Event, ProcessConfig};
use crate::utils::{get_formatted_date, apply_original_dates, is_video};
use crate::ffmpeg::run_ffmpeg;

pub enum Action {
    Rename,
    Reencode { quality: u8 },
    RenameReencode { quality: u8 },
    Lut { path: PathBuf, quality: u8 },
}

pub fn scan_files(source: &Path) -> Vec<PathBuf> {
    WalkDir::new(source)
        .max_depth(1) // On reste au niveau du dossier pour correspondre au script
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && is_video(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn process_batch(files: Vec<PathBuf>, config: &ProcessConfig, action: &Action) -> Result<()> {
    let total = files.len();
    if total == 0 {
        return Err(anyhow!("No video files found in source directory"));
    }

    if !config.dry_run && !config.export.exists() {
        fs::create_dir_all(&config.export)?;
    }

    for (idx, file) in files.iter().enumerate() {
        let current_idx = idx + 1;
        let filename = file.file_name().unwrap().to_string_lossy().to_string();
        let stem = file.file_stem().unwrap().to_string_lossy().to_string();
        
        match action {
            Action::Rename => {
                let prefix = get_formatted_date(file)?;
                if filename.starts_with(&prefix) {
                    log_info(config, &format!("Skipping already renamed file: {}", filename));
                    continue;
                }
                let new_name = format!("{}_{}", prefix, filename);
                let dest = file.with_file_name(&new_name);
                
                log_info(config, &format!("Renaming {} -> {}", filename, new_name));
                if !config.dry_run {
                    fs::rename(file, &dest)?;
                }
            },
            Action::Reencode { quality } => {
                let output = config.export.join(format!("{}.mov", stem));
                if output.exists() && !config.overwrite {
                    log_info(config, &format!("Skipping existing file: {}", output.display()));
                    continue;
                }
                
                let args = vec![
                    "-y".to_string(),
                    "-i".to_string(), file.to_string_lossy().to_string(),
                    "-c:v".to_string(), "hevc_videotoolbox".to_string(),
                    "-profile:v".to_string(), "main10".to_string(),
                    "-pix_fmt".to_string(), "p010le".to_string(),
                    "-q:v".to_string(), quality.to_string(),
                    "-tag:v".to_string(), "hvc1".to_string(),
                    "-c:a".to_string(), "aac".to_string(),
                    "-b:a".to_string(), "192k".to_string(),
                    output.to_string_lossy().to_string()
                ];
                
                run_ffmpeg(args, config, file, current_idx, total)?;
                if !config.dry_run {
                    apply_original_dates(file, &output)?;
                }
                log_file_done(config, &filename, &output.to_string_lossy());
            },
            Action::RenameReencode { quality } => {
                let prefix = get_formatted_date(file)?;
                let output = config.export.join(format!("{}_{}.mov", prefix, stem));
                
                if output.exists() && !config.overwrite {
                    log_info(config, &format!("Skipping existing file: {}", output.display()));
                    continue;
                }

                let args = vec![
                    "-y".to_string(),
                    "-i".to_string(), file.to_string_lossy().to_string(),
                    "-c:v".to_string(), "hevc_videotoolbox".to_string(),
                    "-profile:v".to_string(), "main10".to_string(),
                    "-pix_fmt".to_string(), "p010le".to_string(),
                    "-q:v".to_string(), quality.to_string(),
                    "-tag:v".to_string(), "hvc1".to_string(),
                    "-c:a".to_string(), "aac".to_string(),
                    "-b:a".to_string(), "192k".to_string(),
                    output.to_string_lossy().to_string()
                ];

                run_ffmpeg(args, config, file, current_idx, total)?;
                if !config.dry_run {
                    apply_original_dates(file, &output)?;
                }
                log_file_done(config, &filename, &output.to_string_lossy());
            },
            Action::Lut { path, quality } => {
                let output = config.export.join(format!("{}_lutted.mov", stem));
                if output.exists() && !config.overwrite {
                    log_info(config, &format!("Skipping existing file: {}", output.display()));
                    continue;
                }

                let vf = format!("format=p010le,lut3d='{}'", path.to_string_lossy());
                let args = vec![
                    "-y".to_string(),
                    "-i".to_string(), file.to_string_lossy().to_string(),
                    "-vf".to_string(), vf,
                    "-c:v".to_string(), "hevc_videotoolbox".to_string(),
                    "-profile:v".to_string(), "main10".to_string(),
                    "-pix_fmt".to_string(), "p010le".to_string(),
                    "-q:v".to_string(), quality.to_string(),
                    "-tag:v".to_string(), "hvc1".to_string(),
                    "-c:a".to_string(), "aac".to_string(),
                    "-b:a".to_string(), "192k".to_string(),
                    output.to_string_lossy().to_string()
                ];

                run_ffmpeg(args, config, file, current_idx, total)?;
                if !config.dry_run {
                    apply_original_dates(file, &output)?;
                }
                log_file_done(config, &filename, &output.to_string_lossy());
            }
        }
    }

    if config.jsonl {
        let done = Event::Done { export: config.export.to_string_lossy().to_string() };
        println!("{}", serde_json::to_string(&done)?);
    } else {
        println!("✨ Done! Exported to: {}", config.export.display());
    }

    Ok(())
}

fn log_info(config: &ProcessConfig, msg: &str) {
    if config.jsonl {
        let ev = Event::Log { level: "info".to_string(), message: msg.to_string() };
        if let Ok(s) = serde_json::to_string(&ev) { println!("{}", s); }
    } else {
        println!("  info: {}", msg);
    }
}

fn log_file_done(config: &ProcessConfig, file: &str, output: &str) {
    if config.jsonl {
        let ev = Event::FileDone { file: file.to_string(), output: output.to_string() };
        if let Ok(s) = serde_json::to_string(&ev) { println!("{}", s); }
    } else {
        println!("  ✅ {}", file);
    }
}
