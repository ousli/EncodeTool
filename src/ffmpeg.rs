use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Clone)]
pub struct Tools {
    pub ffmpeg: PathBuf,
    pub ffprobe: Option<PathBuf>,
    pub videotoolbox: bool,
}

fn find_binary(name: &str) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec!["/opt/homebrew/bin".into(), "/usr/local/bin".into()];
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    dirs.into_iter().map(|d| d.join(name)).find(|p| p.is_file())
}

impl Tools {
    pub async fn detect() -> Option<Tools> {
        let ffmpeg = find_binary("ffmpeg")?;
        let ffprobe = find_binary("ffprobe");
        let encoders = Command::new(&ffmpeg)
            .args(["-hide_banner", "-encoders"])
            .output()
            .await
            .ok()?;
        let videotoolbox = String::from_utf8_lossy(&encoders.stdout).contains("hevc_videotoolbox");
        Some(Tools { ffmpeg, ffprobe, videotoolbox })
    }

    /// Media duration in seconds, 0.0 when unknown.
    pub async fn duration(&self, input: &Path) -> f64 {
        let Some(ffprobe) = &self.ffprobe else { return 0.0 };
        let out = Command::new(ffprobe)
            .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1"])
            .arg(input)
            .output()
            .await;
        out.ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<f64>().ok())
            .filter(|d| d.is_finite() && *d > 0.0)
            .unwrap_or(0.0)
    }
}

/// Escapes a path for use inside `-vf lut3d=file=...`: once for the filter
/// option value, once more for the filtergraph description.
pub fn lut_filter(path: &str) -> String {
    let escape = |s: &str, specials: &[char]| {
        let mut out = String::with_capacity(s.len() + 8);
        for c in s.chars() {
            if specials.contains(&c) {
                out.push('\\');
            }
            out.push(c);
        }
        out
    };
    let level1 = escape(path, &['\\', '\'', ':']);
    let level2 = escape(&level1, &['\\', '\'', ',', '[', ']', ';']);
    format!("format=p010le,lut3d=file={level2}")
}

pub struct EncodeParams<'a> {
    pub input: &'a Path,
    pub output: &'a Path,
    pub lut: Option<&'a str>,
}

/// ffmpeg arguments for the H.265 10-bit (VideoToolbox) encode, with or without LUT.
pub fn encode_args(p: &EncodeParams) -> Vec<String> {
    let mut args: Vec<String> = ["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    args.push(p.input.to_string_lossy().into_owned());
    let quality = if let Some(lut) = p.lut {
        args.push("-vf".into());
        args.push(lut_filter(lut));
        "60"
    } else {
        "65"
    };
    for a in [
        "-c:v", "hevc_videotoolbox", "-profile:v", "main10", "-pix_fmt", "p010le", "-q:v", quality, "-tag:v",
        "hvc1", "-c:a", "aac", "-b:a", "192k", "-f", "mov", "-progress", "pipe:1", "-nostats",
    ] {
        args.push(a.into());
    }
    args.push(p.output.to_string_lossy().into_owned());
    args
}

/// Accumulates the key=value blocks written by `ffmpeg -progress pipe:1`.
#[derive(Default, Debug)]
pub struct Progress {
    pub time_us: f64,
    pub speed: Option<f64>,
    pub fps: Option<f64>,
    pub bitrate: Option<String>,
}

impl Progress {
    /// Feeds one line; returns true when it closes a block (`progress=...`).
    pub fn feed(&mut self, line: &str) -> bool {
        let Some((key, value)) = line.split_once('=') else { return false };
        let value = value.trim();
        match key.trim() {
            "out_time_us" | "out_time_ms" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.time_us = v;
                }
            }
            "speed" => self.speed = value.trim_end_matches('x').trim().parse().ok(),
            "fps" => self.fps = value.parse().ok(),
            "bitrate" => self.bitrate = (value != "N/A").then(|| value.to_string()),
            "progress" => return true,
            _ => {}
        }
        false
    }

    pub fn percent(&self, duration: f64) -> f64 {
        if duration <= 0.0 {
            return 0.0;
        }
        (self.time_us / 1_000_000.0 / duration * 100.0).clamp(0.0, 100.0)
    }

    pub fn eta_secs(&self, duration: f64) -> Option<u64> {
        let speed = self.speed.filter(|s| *s > 0.0001)?;
        if duration <= 0.0 {
            return None;
        }
        Some(((duration - self.time_us / 1_000_000.0).max(0.0) / speed) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_progress_block() {
        let mut p = Progress::default();
        for line in ["fps=24.5", "bitrate=1500.2kbits/s", "out_time_us=5000000", "speed=2.5x"] {
            assert!(!p.feed(line));
        }
        assert!(p.feed("progress=continue"));
        assert_eq!(p.percent(10.0), 50.0);
        assert_eq!(p.eta_secs(10.0), Some(2));
        assert_eq!(p.fps, Some(24.5));
        assert_eq!(p.bitrate.as_deref(), Some("1500.2kbits/s"));
    }

    #[test]
    fn ignores_unknown_values() {
        let mut p = Progress::default();
        p.feed("out_time_us=N/A");
        p.feed("speed=N/A");
        p.feed("bitrate=N/A");
        assert_eq!(p.percent(0.0), 0.0);
        assert_eq!(p.eta_secs(10.0), None);
        assert_eq!(p.bitrate, None);
    }

    #[test]
    fn escapes_lut_path() {
        assert_eq!(lut_filter("/a b/c.cube"), "format=p010le,lut3d=file=/a b/c.cube");
        assert_eq!(lut_filter("/a:b.cube"), "format=p010le,lut3d=file=/a\\\\:b.cube");
        assert_eq!(lut_filter("/it's,[x].cube"), "format=p010le,lut3d=file=/it\\\\\\'s\\,\\[x\\].cube");
    }
}
