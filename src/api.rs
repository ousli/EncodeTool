use crate::db::Db;
use crate::fsops;
use crate::jobs::{Job, Kind, Manager, NewJob};
use axum::extract::{Path, Query, Request, State};
use axum::http::{header, StatusCode, Uri};
use axum::middleware::{self, Next};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use futures_util::stream::{self, Stream, StreamExt};
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;

#[derive(RustEmbed)]
#[folder = "web/dist"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub jobs: Arc<Manager>,
}

pub struct ApiError(StatusCode, String);

impl ApiError {
    fn bad(msg: impl Into<String>) -> Self {
        ApiError(StatusCode::BAD_REQUEST, msg.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

fn hostname(value: &str) -> &str {
    let value = value.trim_start_matches("http://").trim_start_matches("https://");
    let host = value.split('/').next().unwrap_or("");
    if host.starts_with('[') {
        return host.split(']').next().map(|h| &h[1..]).unwrap_or("");
    }
    host.split(':').next().unwrap_or("")
}

fn is_local(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1")
}

/// Blocks DNS-rebinding and cross-site requests: the API drives ffmpeg and
/// browses the disk, so only pages served by this very app may call it.
async fn local_only(req: Request, next: Next) -> Response {
    let allowed = {
        let host = |name: header::HeaderName| req.headers().get(name).and_then(|v| v.to_str().ok()).map(|v| is_local(hostname(v)));
        host(header::HOST) == Some(true) && host(header::ORIGIN).unwrap_or(true)
    };
    if allowed {
        next.run(req).await
    } else {
        (StatusCode::FORBIDDEN, Json(json!({ "error": "Accès refusé" }))).into_response()
    }
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route("/jobs", get(list_jobs).post(create_job).delete(clear_jobs))
        .route("/jobs/:id", delete(remove_job))
        .route("/jobs/:id/cancel", post(cancel_job))
        .route("/jobs/:id/pause", post(pause_job))
        .route("/jobs/:id/resume", post(resume_job))
        .route("/events", get(events))
        .route("/luts", get(list_luts).post(add_lut))
        .route("/luts/:id", delete(delete_lut))
        .route("/fs/browse", get(browse))
        .route("/fs/pick", post(pick));
    Router::new()
        .nest("/api", api)
        .fallback(static_file)
        .layer(middleware::from_fn(local_only))
        .with_state(state)
}

async fn static_file(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    let (path, file) = match Assets::get(path) {
        Some(file) => (path, file),
        None => match Assets::get("index.html") {
            Some(file) => ("index.html", file),
            None => return (StatusCode::NOT_FOUND, "Frontend non compilé : lancez ./build.sh").into_response(),
        },
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    ([(header::CONTENT_TYPE, mime.as_ref().to_string())], file.data.into_owned()).into_response()
}

async fn health(State(s): State<AppState>) -> Json<serde_json::Value> {
    let tools = s.jobs.tools();
    Json(json!({
        "ffmpeg": tools.is_some(),
        "ffprobe": tools.is_some_and(|t| t.ffprobe.is_some()),
        "videotoolbox": tools.is_some_and(|t| t.videotoolbox),
    }))
}

async fn list_jobs(State(s): State<AppState>) -> Json<Vec<Job>> {
    Json(s.jobs.snapshot())
}

#[derive(Deserialize)]
struct CreateJob {
    kind: Kind,
    source: String,
    export: Option<String>,
    lut_id: Option<i64>,
}

async fn create_job(State(s): State<AppState>, Json(body): Json<CreateJob>) -> ApiResult<Job> {
    let source = fsops::clean_path(&body.source);
    if !PathBuf::from(&source).is_dir() {
        return Err(ApiError::bad(format!("Dossier source invalide : {source}")));
    }
    let mut export = None;
    let (mut lut_name, mut lut_path) = (None, None);
    if body.kind != Kind::Rename {
        if s.jobs.tools().is_none() {
            return Err(ApiError::bad("ffmpeg n'est pas installé"));
        }
        let raw = body.export.as_deref().map(fsops::clean_path).filter(|e| !e.is_empty());
        let dir = raw.unwrap_or_else(|| format!("{}/export", source.trim_end_matches('/')));
        std::fs::create_dir_all(&dir).map_err(|e| ApiError::bad(format!("Dossier d'export invalide : {e}")))?;
        export = Some(dir);
    }
    if body.kind == Kind::Lut {
        let id = body.lut_id.ok_or_else(|| ApiError::bad("Choisissez une LUT"))?;
        let lut = s.db.get_lut(id).ok().flatten().ok_or_else(|| ApiError::bad("LUT introuvable"))?;
        if !PathBuf::from(&lut.path).is_file() {
            return Err(ApiError::bad(format!("Fichier LUT introuvable : {}", lut.path)));
        }
        lut_name = Some(lut.name);
        lut_path = Some(lut.path);
    }
    Ok(Json(s.jobs.enqueue(NewJob { kind: body.kind, source, export, lut_name, lut_path })))
}

async fn cancel_job(State(s): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    s.jobs.cancel(id).map_err(ApiError::bad)?;
    Ok(Json(json!({ "ok": true })))
}

async fn pause_job(State(s): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    s.jobs.pause(id).map_err(ApiError::bad)?;
    Ok(Json(json!({ "ok": true })))
}

async fn resume_job(State(s): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    s.jobs.resume(id).map_err(ApiError::bad)?;
    Ok(Json(json!({ "ok": true })))
}

async fn remove_job(State(s): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    s.jobs.remove(id).map_err(ApiError::bad)?;
    Ok(Json(json!({ "ok": true })))
}

async fn clear_jobs(State(s): State<AppState>) -> Json<serde_json::Value> {
    s.jobs.clear_finished();
    Json(json!({ "ok": true }))
}

async fn events(State(s): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let changes = BroadcastStream::new(s.jobs.subscribe()).map(|_| ());
    let stream = stream::iter([()]).chain(changes).map(move |_| {
        let event = Event::default().json_data(s.jobs.snapshot()).unwrap_or_default();
        Ok(event)
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn list_luts(State(s): State<AppState>) -> ApiResult<Vec<crate::db::Lut>> {
    s.db.list_luts().map(Json).map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Deserialize)]
struct NewLut {
    name: String,
    path: String,
}

async fn add_lut(State(s): State<AppState>, Json(body): Json<NewLut>) -> ApiResult<crate::db::Lut> {
    let name = body.name.trim();
    let path = fsops::clean_path(&body.path);
    if name.is_empty() {
        return Err(ApiError::bad("Le nom est obligatoire"));
    }
    if !PathBuf::from(&path).is_file() || !path.to_lowercase().ends_with(".cube") {
        return Err(ApiError::bad(format!("Fichier .cube introuvable : {path}")));
    }
    s.db.add_lut(name, &path).map(Json).map_err(|_| ApiError::bad("Une LUT porte déjà ce nom"))
}

async fn delete_lut(State(s): State<AppState>, Path(id): Path<i64>) -> ApiResult<serde_json::Value> {
    match s.db.delete_lut(id) {
        Ok(true) => Ok(Json(json!({ "ok": true }))),
        _ => Err(ApiError(StatusCode::NOT_FOUND, "LUT introuvable".into())),
    }
}

#[derive(Deserialize)]
struct BrowseQuery {
    path: Option<String>,
    hidden: Option<bool>,
}

async fn browse(Query(q): Query<BrowseQuery>) -> ApiResult<fsops::Browse> {
    fsops::browse(q.path.as_deref().unwrap_or(""), q.hidden.unwrap_or(false)).map(Json).map_err(ApiError::bad)
}

async fn pick() -> ApiResult<serde_json::Value> {
    let path = fsops::pick_folder().await.map_err(ApiError::bad)?;
    Ok(Json(json!({ "path": path })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_hostnames() {
        assert_eq!(hostname("127.0.0.1:7878"), "127.0.0.1");
        assert_eq!(hostname("http://localhost:5173"), "localhost");
        assert_eq!(hostname("[::1]:7878"), "::1");
        assert!(!is_local(hostname("http://evil.com")));
        assert!(!is_local(hostname("evil.com:80")));
    }
}
