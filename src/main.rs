mod api;
mod db;
mod ffmpeg;
mod fsops;
mod jobs;

use std::path::PathBuf;
use std::sync::Arc;

fn data_dir() -> PathBuf {
    let dir = std::env::var_os("ENCODETOOL_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(fsops::home()).join("Library/Application Support/EncodeTool"));
    std::fs::create_dir_all(&dir).expect("impossible de créer le dossier de données");
    dir
}

#[tokio::main]
async fn main() {
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

    let app = api::router(api::AppState { db, jobs });
    let mut listener = None;
    for p in port..port.saturating_add(10) {
        if let Ok(l) = tokio::net::TcpListener::bind(("127.0.0.1", p)).await {
            listener = Some(l);
            break;
        }
    }
    let listener = listener.expect("aucun port disponible");
    let url = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    println!("🎬 EncodeTool : {url}  (Ctrl+C pour quitter)");
    if !no_open {
        let _ = std::process::Command::new("open").arg(&url).spawn();
    }
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .unwrap();
}
