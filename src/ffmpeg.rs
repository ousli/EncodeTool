use anyhow::{Result, anyhow};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use crate::models::{Event, ProcessConfig};

pub fn get_duration(path: &Path) -> Result<f64> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("ffprobe failed"));
    }

    let s = String::from_utf8(output.stdout)?;
    let duration: f64 = s.trim().parse()?;
    Ok(duration)
}

pub fn run_ffmpeg(
    args: Vec<String>,
    config: &ProcessConfig,
    file_path: &Path,
    file_index: usize,
    file_total: usize,
) -> Result<()> {
    let filename = file_path.file_name().unwrap().to_string_lossy().to_string();
    let duration = get_duration(file_path).unwrap_or(0.0);
    let duration_us = (duration * 1_000_000.0) as i64;

    if config.dry_run {
        println!("  [DRY RUN] ffmpeg {}", args.join(" "));
        return Ok(());
    }

    let mut child = Command::new("ffmpeg")
        .args(args)
        .arg("-progress")
        .arg("pipe:1")
        .arg("-nostats")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    let mut out_time_us: i64 = 0;
    let mut speed = String::from("0x");

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() < 2 { continue; }
        let key = parts[0];
        let value = parts[1];

        match key {
            "out_time_us" => out_time_us = value.parse().unwrap_or(0),
            "speed" => speed = value.to_string(),
            "progress" => {
                let file_percent = if duration_us > 0 {
                    (out_time_us as f64 / duration_us as f64 * 100.0).clamp(0.0, 100.0)
                } else {
                    0.0
                };
                
                let global_percent = ((file_index - 1) as f64 * 100.0 + file_percent) / file_total as f64;
                
                let event = Event::Progress {
                    file: filename.clone(),
                    file_index,
                    file_total,
                    file_percent,
                    global_percent,
                    eta: "".to_string(), // On pourrait calculer l'ETA via le speed ici
                };

                if config.jsonl {
                    println!("{}", serde_json::to_string(&event)?);
                } else {
                    // Affichage simple sur une ligne si pas en JSONL
                    print!("\r  [{}/{}] {} : {:.1}% (Vitesse: {})    ", 
                        file_index, file_total, filename, file_percent, speed);
                    use std::io::Write;
                    std::io::stdout().flush()?;
                }
            }
            _ => {}
        }
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!("ffmpeg exited with error status"));
    }
    
    if !config.jsonl {
        println!(); // Fin de ligne pour le \r
    }

    Ok(())
}
