use crate::db::Db;
use crate::ffmpeg::{encode_args, EncodeParams, Progress, Tools};
use crate::fsops;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{broadcast, Notify};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Rename,
    Encode,
    RenameEncode,
    Lut,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Queued,
    Running,
    Paused,
    Done,
    Failed,
    Cancelled,
}

impl Status {
    pub fn is_finished(self) -> bool {
        matches!(self, Status::Done | Status::Failed | Status::Cancelled)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Job {
    pub id: i64,
    pub kind: Kind,
    pub source: String,
    pub export: Option<String>,
    pub lut_name: Option<String>,
    pub lut_path: Option<String>,
    #[serde(default = "yes")]
    pub keep_metadata: bool,
    pub status: Status,
    pub total: usize,
    pub index: usize,
    pub done_files: usize,
    pub current_file: Option<String>,
    pub file_percent: f64,
    pub percent: f64,
    pub eta_secs: Option<u64>,
    pub fps: Option<f64>,
    pub bitrate: Option<String>,
    pub speed: Option<f64>,
    pub error: Option<String>,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    /// Bumped each time new preview frames are available (0 = none).
    #[serde(default)]
    pub thumb_v: u32,
}

fn yes() -> bool {
    true
}

pub struct NewJob {
    pub kind: Kind,
    pub source: String,
    pub export: Option<String>,
    pub lut_name: Option<String>,
    pub lut_path: Option<String>,
    pub keep_metadata: bool,
}

struct RunCtl {
    id: i64,
    pid: Option<u32>,
    cancel: bool,
    paused: bool,
}

enum Stop {
    Cancelled,
    Failed(String),
}

pub struct Manager {
    db: Arc<Db>,
    tools: Option<Tools>,
    jobs: Mutex<Vec<Job>>,
    next_id: AtomicI64,
    ctl: Mutex<Option<RunCtl>>,
    changed: broadcast::Sender<()>,
    wake: Notify,
    thumbs: Mutex<HashMap<i64, (Vec<u8>, Option<Vec<u8>>)>>,
    shutting_down: AtomicBool,
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn signal(pid: u32, sig: i32) {
    // SAFETY: plain kill(2) on a child pid we spawned.
    unsafe {
        libc::kill(pid as i32, sig);
    }
}

fn tail(text: &str, max: usize) -> String {
    let chars: Vec<char> = text.trim().chars().collect();
    chars[chars.len().saturating_sub(max)..].iter().collect()
}

/// Output file name for a video (without directory).
pub fn output_name(kind: Kind, stem: &str, date_prefix: &str, lut_name: Option<&str>) -> String {
    let lut_part = lut_name.map(|name| {
        let profile: String = name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
        format!("_{profile}_lutted")
    });
    let lut_part = lut_part.unwrap_or_default();
    match kind {
        Kind::RenameEncode => format!("{date_prefix}_{stem}{lut_part}.mov"),
        _ => format!("{stem}{lut_part}.mov"),
    }
}

impl Manager {
    pub fn new(db: Arc<Db>, tools: Option<Tools>) -> Arc<Manager> {
        let mut jobs = db.load_jobs();
        for job in jobs.iter_mut().filter(|j| !j.status.is_finished()) {
            job.status = Status::Failed;
            job.error = Some("Interrompue : l'application a été arrêtée".into());
            job.finished_at = Some(now());
            job.eta_secs = None;
            db.save_job(job);
        }
        let next_id = jobs.iter().map(|j| j.id).max().unwrap_or(0) + 1;
        Arc::new(Manager {
            db,
            tools,
            jobs: Mutex::new(jobs),
            next_id: AtomicI64::new(next_id),
            ctl: Mutex::new(None),
            changed: broadcast::channel(64).0,
            wake: Notify::new(),
            thumbs: Mutex::new(HashMap::new()),
            shutting_down: AtomicBool::new(false),
        })
    }

    pub fn tools(&self) -> Option<&Tools> {
        self.tools.as_ref()
    }

    pub fn snapshot(&self) -> Vec<Job> {
        self.jobs.lock().unwrap().clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.changed.subscribe()
    }

    pub fn enqueue(&self, new: NewJob) -> Job {
        let job = Job {
            id: self.next_id.fetch_add(1, Ordering::SeqCst),
            kind: new.kind,
            source: new.source,
            export: new.export,
            lut_name: new.lut_name,
            lut_path: new.lut_path,
            keep_metadata: new.keep_metadata,
            status: Status::Queued,
            total: 0,
            index: 0,
            done_files: 0,
            current_file: None,
            file_percent: 0.0,
            percent: 0.0,
            eta_secs: None,
            fps: None,
            bitrate: None,
            speed: None,
            error: None,
            created_at: now(),
            started_at: None,
            finished_at: None,
            thumb_v: 0,
        };
        self.db.save_job(&job);
        self.jobs.lock().unwrap().push(job.clone());
        let _ = self.changed.send(());
        self.wake.notify_one();
        job
    }

    fn update(&self, id: i64, f: impl FnOnce(&mut Job)) {
        if let Some(job) = self.jobs.lock().unwrap().iter_mut().find(|j| j.id == id) {
            f(job);
        }
        let _ = self.changed.send(());
    }

    fn persist(&self, id: i64) {
        let job = self.jobs.lock().unwrap().iter().find(|j| j.id == id).cloned();
        if let Some(job) = job {
            self.db.save_job(&job);
        }
    }

    fn status_of(&self, id: i64) -> Option<Status> {
        self.jobs.lock().unwrap().iter().find(|j| j.id == id).map(|j| j.status)
    }

    pub fn cancel(&self, id: i64) -> Result<(), String> {
        match self.status_of(id).ok_or("Tâche introuvable")? {
            Status::Queued => {
                self.update(id, |j| {
                    j.status = Status::Cancelled;
                    j.finished_at = Some(now());
                });
                self.persist(id);
                Ok(())
            }
            Status::Running | Status::Paused => {
                if let Some(ctl) = self.ctl.lock().unwrap().as_mut().filter(|c| c.id == id) {
                    ctl.cancel = true;
                    if let Some(pid) = ctl.pid {
                        signal(pid, libc::SIGKILL);
                    }
                }
                Ok(())
            }
            _ => Err("La tâche est déjà terminée".into()),
        }
    }

    pub fn pause(&self, id: i64) -> Result<(), String> {
        if self.status_of(id) != Some(Status::Running) {
            return Err("Seule une tâche en cours peut être mise en pause".into());
        }
        if let Some(ctl) = self.ctl.lock().unwrap().as_mut().filter(|c| c.id == id) {
            ctl.paused = true;
            if let Some(pid) = ctl.pid {
                signal(pid, libc::SIGSTOP);
            }
        }
        self.update(id, |j| {
            j.status = Status::Paused;
            j.eta_secs = None;
        });
        Ok(())
    }

    pub fn resume(&self, id: i64) -> Result<(), String> {
        if self.status_of(id) != Some(Status::Paused) {
            return Err("La tâche n'est pas en pause".into());
        }
        if let Some(ctl) = self.ctl.lock().unwrap().as_mut().filter(|c| c.id == id) {
            ctl.paused = false;
            if let Some(pid) = ctl.pid {
                signal(pid, libc::SIGCONT);
            }
        }
        self.update(id, |j| j.status = Status::Running);
        Ok(())
    }

    pub fn remove(&self, id: i64) -> Result<(), String> {
        let mut jobs = self.jobs.lock().unwrap();
        let pos = jobs.iter().position(|j| j.id == id).ok_or("Tâche introuvable")?;
        if !jobs[pos].status.is_finished() {
            return Err("Annulez la tâche avant de la supprimer".into());
        }
        jobs.remove(pos);
        drop(jobs);
        self.db.delete_job(id);
        let _ = self.changed.send(());
        Ok(())
    }

    pub fn clear_finished(&self) {
        let removed: Vec<i64> = {
            let mut jobs = self.jobs.lock().unwrap();
            let ids = jobs.iter().filter(|j| j.status.is_finished()).map(|j| j.id).collect();
            jobs.retain(|j| !j.status.is_finished());
            ids
        };
        for id in removed {
            self.db.delete_job(id);
        }
        let _ = self.changed.send(());
    }

    /// Preview frames of the running job: `lut = false` is the original frame.
    pub fn thumb(&self, id: i64, lut: bool) -> Option<Vec<u8>> {
        let thumbs = self.thumbs.lock().unwrap();
        let (orig, graded) = thumbs.get(&id)?;
        if lut { graded.clone() } else { Some(orig.clone()) }
    }

    /// Stops the running ffmpeg (removing its partial file) and prevents new jobs from starting.
    pub async fn shutdown(&self) {
        self.shutting_down.store(true, Ordering::SeqCst);
        if let Some(ctl) = self.ctl.lock().unwrap().as_mut() {
            ctl.cancel = true;
            if let Some(pid) = ctl.pid {
                signal(pid, libc::SIGKILL);
            }
        }
        for _ in 0..30 {
            if self.ctl.lock().unwrap().is_none() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Runs queued jobs one at a time, forever.
    pub async fn worker(self: Arc<Self>) {
        loop {
            match self.take_next() {
                Some(job) => self.run_job(job).await,
                None => self.wake.notified().await,
            }
        }
    }

    fn take_next(&self) -> Option<Job> {
        if self.shutting_down.load(Ordering::SeqCst) {
            return None;
        }
        let job = {
            let mut jobs = self.jobs.lock().unwrap();
            let job = jobs.iter_mut().find(|j| j.status == Status::Queued)?;
            job.status = Status::Running;
            job.started_at = Some(now());
            job.clone()
        };
        *self.ctl.lock().unwrap() = Some(RunCtl { id: job.id, pid: None, cancel: false, paused: false });
        self.persist(job.id);
        let _ = self.changed.send(());
        Some(job)
    }

    async fn run_job(&self, job: Job) {
        let result = self.process(&job).await;
        let id = job.id;
        self.update(id, |j| {
            match &result {
                Ok(()) => {
                    j.status = Status::Done;
                    j.percent = 100.0;
                    j.file_percent = 100.0;
                }
                Err(Stop::Cancelled) => j.status = Status::Cancelled,
                Err(Stop::Failed(msg)) => {
                    j.status = Status::Failed;
                    j.error = Some(msg.clone());
                }
            }
            j.finished_at = Some(now());
            j.eta_secs = None;
            j.fps = None;
            j.speed = None;
            j.current_file = None;
        });
        self.thumbs.lock().unwrap().remove(&id);
        *self.ctl.lock().unwrap() = None;
        self.persist(id);
    }

    /// Waits while the job is paused; errors out when it was cancelled.
    async fn gate(&self, id: i64) -> Result<(), Stop> {
        loop {
            {
                let ctl = self.ctl.lock().unwrap();
                match ctl.as_ref().filter(|c| c.id == id) {
                    Some(c) if c.cancel => return Err(Stop::Cancelled),
                    Some(c) if c.paused => {}
                    _ => return Ok(()),
                }
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    async fn process(&self, job: &Job) -> Result<(), Stop> {
        let id = job.id;
        let source = PathBuf::from(&job.source);
        let files = fsops::list_videos(&source).map_err(|e| Stop::Failed(format!("Lecture du dossier impossible : {e}")))?;
        if files.is_empty() {
            return Err(Stop::Failed("Aucune vidéo (mp4/mov) trouvée dans le dossier".into()));
        }
        let total = files.len();
        self.update(id, |j| j.total = total);
        for (i, file) in files.iter().enumerate() {
            self.gate(id).await?;
            let name = file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            self.update(id, |j| {
                j.index = i + 1;
                j.current_file = Some(name.clone());
                j.file_percent = 0.0;
                j.eta_secs = None;
            });
            if job.kind == Kind::Rename {
                fsops::rename_with_date(file).map_err(|e| Stop::Failed(format!("{name} : {e}")))?;
            } else {
                self.encode_one(job, file, i + 1, total).await?;
            }
            self.update(id, |j| {
                j.done_files = i + 1;
                j.file_percent = 100.0;
                j.percent = (i + 1) as f64 * 100.0 / total as f64;
            });
        }
        Ok(())
    }

    async fn encode_one(&self, job: &Job, file: &Path, idx: usize, total: usize) -> Result<(), Stop> {
        let tools = self.tools.as_ref().ok_or_else(|| Stop::Failed("ffmpeg est introuvable".into()))?;
        let export = PathBuf::from(job.export.as_deref().unwrap_or(&job.source));
        let stem = file.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let prefix = if job.kind == Kind::RenameEncode {
            fsops::date_prefix(file).map_err(|e| Stop::Failed(e.to_string()))?
        } else {
            String::new()
        };
        let output = export.join(output_name(job.kind, &stem, &prefix, job.lut_name.as_deref()));
        if output == file {
            return Err(Stop::Failed(format!(
                "Le fichier de sortie écraserait la source ({}) : choisissez un autre dossier d'export",
                file.display()
            )));
        }
        let partial = export.join(format!(".{}.part", output.file_name().unwrap().to_string_lossy()));
        let duration = tools.duration(file).await;
        self.make_thumbs(job, tools, file, duration).await;
        let args = encode_args(&EncodeParams { input: file, output: &partial, lut: job.lut_path.as_deref(), keep_metadata: job.keep_metadata });
        let result = self.run_ffmpeg(job.id, tools, &args, duration, idx, total).await;
        match result {
            Ok(()) => {
                std::fs::rename(&partial, &output).map_err(|e| Stop::Failed(format!("Écriture de {} impossible : {e}", output.display())))?;
                let _ = fsops::copy_dates(file, &output);
                Ok(())
            }
            Err(e) => {
                let _ = std::fs::remove_file(&partial);
                Err(e)
            }
        }
    }

    async fn make_thumbs(&self, job: &Job, tools: &Tools, file: &Path, duration: f64) {
        let at = duration * 0.1;
        let lut = job.lut_path.as_deref();
        let (orig, graded) = tokio::join!(tools.thumbnail(file, at, None), async {
            match lut {
                Some(l) => tools.thumbnail(file, at, Some(l)).await,
                None => None,
            }
        });
        let id = job.id;
        match orig {
            Some(orig) => {
                self.thumbs.lock().unwrap().insert(id, (orig, graded));
                self.update(id, |j| j.thumb_v += 1);
            }
            None => {
                self.thumbs.lock().unwrap().remove(&id);
            }
        }
    }

    async fn run_ffmpeg(&self, id: i64, tools: &Tools, args: &[String], duration: f64, idx: usize, total: usize) -> Result<(), Stop> {
        let mut child = Command::new(&tools.ffmpeg)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| Stop::Failed(format!("Lancement de ffmpeg impossible : {e}")))?;
        if let Some(pid) = child.id() {
            if let Some(ctl) = self.ctl.lock().unwrap().as_mut().filter(|c| c.id == id) {
                ctl.pid = Some(pid);
                if ctl.cancel {
                    signal(pid, libc::SIGKILL);
                } else if ctl.paused {
                    signal(pid, libc::SIGSTOP);
                }
            }
        }
        let mut stderr = child.stderr.take().expect("piped stderr");
        let stderr_task = tokio::spawn(async move {
            let mut buf = String::new();
            let _ = stderr.read_to_string(&mut buf).await;
            buf
        });
        let mut lines = BufReader::new(child.stdout.take().expect("piped stdout")).lines();
        let mut progress = Progress::default();
        while let Ok(Some(line)) = lines.next_line().await {
            if !progress.feed(&line) {
                continue;
            }
            let file_percent = progress.percent(duration);
            let eta = progress.eta_secs(duration);
            self.update(id, |j| {
                if j.status == Status::Paused {
                    return;
                }
                j.file_percent = file_percent;
                j.percent = ((idx - 1) as f64 * 100.0 + file_percent) / total as f64;
                j.eta_secs = eta;
                j.fps = progress.fps;
                j.bitrate = progress.bitrate.clone();
                j.speed = progress.speed;
            });
        }
        let status = child.wait().await;
        let stderr_text = stderr_task.await.unwrap_or_default();
        let cancelled = {
            let mut ctl = self.ctl.lock().unwrap();
            ctl.as_mut().filter(|c| c.id == id).map(|c| {
                c.pid = None;
                c.cancel
            })
        };
        if cancelled == Some(true) {
            return Err(Stop::Cancelled);
        }
        match status {
            Ok(s) if s.success() => Ok(()),
            Ok(_) => Err(Stop::Failed(format!("ffmpeg a échoué : {}", tail(&stderr_text, 1500)))),
            Err(e) => Err(Stop::Failed(format!("ffmpeg : {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_outputs() {
        assert_eq!(output_name(Kind::Encode, "clip", "", None), "clip.mov");
        assert_eq!(output_name(Kind::RenameEncode, "clip", "2024-01-02_1030", None), "2024-01-02_1030_clip.mov");
        assert_eq!(output_name(Kind::Lut, "clip", "", Some("S-Log 2")), "clip_S-Log-2_lutted.mov");
        assert_eq!(output_name(Kind::RenameEncode, "clip", "2024-01-02_1030", Some("S-Log2")), "2024-01-02_1030_clip_S-Log2_lutted.mov");
        assert_eq!(output_name(Kind::Encode, "clip", "", Some("S-Log2")), "clip_S-Log2_lutted.mov");
    }
}
