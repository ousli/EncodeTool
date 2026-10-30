mod api;
#[cfg(target_os = "macos")]
mod gui;
mod db;
mod ffmpeg;
mod fsops;
mod jobs;

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

static QUIT: tokio::sync::Notify = tokio::sync::Notify::const_new();
static URL: OnceLock<String> = OnceLock::new();

pub fn request_quit() {
    QUIT.notify_one();
}

pub fn open_browser() {
    if let Some(url) = URL.get() {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
}

/// Launched from EncodeTool.app (not from a terminal).
fn in_app_bundle() -> bool {
    std::env::current_exe().is_ok_and(|p| p.to_string_lossy().contains(".app/Contents/MacOS"))
}

fn data_dir() -> PathBuf {
    let dir = std::env::var_os("ENCODETOOL_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(fsops::home()).join("Library/Application Support/EncodeTool"));
    std::fs::create_dir_all(&dir).expect("impossible de créer le dossier de données");
    dir
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    #[cfg(target_os = "macos")]
    if in_app_bundle() {
        std::thread::spawn(move || {
            runtime.block_on(serve());
            std::process::exit(0);
        });
        gui::run();
        return;
    }
    runtime.block_on(serve());
}

async fn serve() {
    let args: Vec<String> = std::env::args().collect();
    let no_open = args.iter().any(|a| a == "--no-open");
    let port: u16 = args
        .windows(2)
        .find(|w| w[0] == "--port")
        .and_then(|w| w[1].parse().ok())
        .or_else(|| std::env::var("ENCODETOOL_PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(7878);

    let db = Arc::new(db::Db::open(&data_dir().join("encodetool.db")).expect("base de données inaccessible"));
    let tools = ffmpeg::Tools::detect().await;
    match &tools {
        None => eprintln!("⚠️  ffmpeg est introuvable : seul le renommage sera disponible."),
        Some(t) if !t.videotoolbox => eprintln!("⚠️  ffmpeg ne supporte pas hevc_videotoolbox."),
        _ => {}
    }
    let jobs = jobs::Manager::new(db.clone(), tools);
    tokio::spawn(jobs.clone().worker());

    let app = api::router(api::AppState { db, jobs: jobs.clone(), quit: &QUIT });
    let mut listener = None;
    for p in port..port.saturating_add(10) {
        if let Ok(l) = tokio::net::TcpListener::bind(("127.0.0.1", p)).await {
            listener = Some(l);
            break;
        }
    }
    let listener = listener.expect("aucun port disponible");
    let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let _ = URL.set(url.clone());
    println!("🎬 EncodeTool : {url}  (Ctrl+C pour quitter)");
    if !no_open {
        open_browser();
    }
    // Long-lived SSE connections would block a graceful shutdown, so stop ffmpeg and exit directly.
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
    tokio::select! {
        res = axum::serve(listener, app) => res.unwrap(),
        _ = tokio::signal::ctrl_c() => {
            println!("\nArrêt en cours…");
            jobs.shutdown().await;
        }
        _ = sigterm.recv() => jobs.shutdown().await,
        _ = QUIT.notified() => jobs.shutdown().await,
    }
}
