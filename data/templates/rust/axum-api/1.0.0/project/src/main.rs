mod domain;
mod repository;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use domain::*;
use repository::Repository;
use serde::Deserialize;
use serde_json::json;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
type App = Arc<Mutex<Repository>>;
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Missing => (StatusCode::NOT_FOUND, "Issue not found".into()),
            Self::Conflict => (
                StatusCode::CONFLICT,
                "Version conflict; reload before editing".into(),
            ),
            Self::Invalid(s) => (StatusCode::UNPROCESSABLE_ENTITY, s),
            Self::Storage(s) => {
                eprintln!("repository error: {s}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unable to read or save data. No successful update was returned.".into(),
                )
            }
        };
        (
            status,
            Json(json!({"error":{"status":status.as_u16(),"message":message}})),
        )
            .into_response()
    }
}
async fn access<T: Send + 'static>(
    app: App,
    f: impl FnOnce(&mut Repository) -> Result<T, Error> + Send + 'static,
) -> Result<T, Error> {
    tokio::task::spawn_blocking(move || {
        let mut repo = app
            .lock()
            .map_err(|_| Error::Storage("repository lock poisoned".into()))?;
        f(&mut repo)
    })
    .await
    .map_err(|e| Error::Storage(e.to_string()))?
}
fn body<T>(value: Result<Json<T>, JsonRejection>) -> Result<T, Error> {
    value.map(|Json(v)| v).map_err(|_| {
        Error::Invalid("Send a valid JSON object with documented fields (maximum 32 KiB)".into())
    })
}
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Filter {
    status: Option<Status>,
    #[serde(default)]
    q: String,
    limit: Option<usize>,
    #[serde(default)]
    offset: usize,
}
async fn list(
    State(app): State<App>,
    Query(filter): Query<Filter>,
) -> Result<Json<serde_json::Value>, Error> {
    let limit = filter.limit.unwrap_or(50);
    if !(1..=100).contains(&limit) {
        return Err(Error::Invalid("limit must be 1–100".into()));
    }
    let rows = access(app, move |r| Ok(r.list(filter.status, &filter.q))).await?;
    let total = rows.len();
    let offset = filter.offset.min(total);
    let end = offset.saturating_add(limit).min(total);
    Ok(Json(
        json!({"items":rows[offset..end],"total":total,"limit":limit,"offset":offset}),
    ))
}
async fn show(State(app): State<App>, Path(id): Path<u64>) -> Result<Json<Issue>, Error> {
    access(app, move |r| r.get(id)).await.map(Json)
}
async fn create(
    State(app): State<App>,
    input: Result<Json<Input>, JsonRejection>,
) -> Result<(StatusCode, Json<Issue>), Error> {
    let input = body(input)?;
    let issue = access(app, move |r| r.create(input)).await?;
    Ok((StatusCode::CREATED, Json(issue)))
}
async fn update(
    State(app): State<App>,
    Path(id): Path<u64>,
    input: Result<Json<Input>, JsonRejection>,
) -> Result<Json<Issue>, Error> {
    let input = body(input)?;
    access(app, move |r| r.update(id, input)).await.map(Json)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Comment {
    version: u64,
    body: String,
}
async fn comment(
    State(app): State<App>,
    Path(id): Path<u64>,
    input: Result<Json<Comment>, JsonRejection>,
) -> Result<Json<Issue>, Error> {
    let input = body(input)?;
    access(app, move |r| r.comment(id, input.version, input.body))
        .await
        .map(Json)
}
fn router(repo: Repository) -> Router {
    Router::new().route("/",get(||async{Json(json!({"app":"Issue Desk","issues":"/issues","workflow":"open → doing → done → open"}))}))
 .route("/healthz",get(||async{StatusCode::OK})).route("/issues",get(list).post(create)).route("/issues/{id}",get(show).put(update)).route("/issues/{id}/comments",axum::routing::post(comment))
 .layer(DefaultBodyLimit::max(32768)).with_state(Arc::new(Mutex::new(repo)))
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = std::env::var("AXUM_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1:8082".into())
        .parse()?;
    if !address.ip().is_loopback() {
        return Err("Use a loopback address for this single-user application".into());
    }
    let path = std::env::var("ISSUE_DATA").unwrap_or_else(|_| ".data/issues.json".into());
    let repo = Repository::open(path).map_err(|e| format!("Cannot open repository: {e:?}"))?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    eprintln!("Issue Desk: http://{address}");
    axum::serve(listener, router(repo)).await?;
    Ok(())
}
#[cfg(test)]
mod http_tests {
    use super::*;
    use std::io::{Read, Write};
    fn request(address: SocketAddr, method: &str, path: &str, body: &str) -> String {
        let mut c = std::net::TcpStream::connect(address).unwrap();
        c.set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        write!(c,"{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",body.len()).unwrap();
        let mut out = String::new();
        c.read_to_string(&mut out).unwrap();
        out
    }
    #[tokio::test]
    async fn complete_http_workflow() {
        let dir = std::env::temp_dir().join(format!("issue-http-{}-{}", std::process::id(), now()));
        let repo = Repository::open(dir.join("issues.json")).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, router(repo)).await });
        let result = tokio::task::spawn_blocking(move || {
            assert!(
                request(
                    address,
                    "POST",
                    "/issues",
                    r#"{"title":"Fix sync","priority":"high"}"#
                )
                .starts_with("HTTP/1.1 201")
            );
            assert!(
                request(
                    address,
                    "PUT",
                    "/issues/1",
                    r#"{"title":"Fix sync","priority":"high","status":"doing","version":1}"#
                )
                .starts_with("HTTP/1.1 200")
            );
            assert!(
                request(
                    address,
                    "POST",
                    "/issues/1/comments",
                    r#"{"version":1,"body":"stale"}"#
                )
                .starts_with("HTTP/1.1 409")
            );
            assert!(
                request(
                    address,
                    "POST",
                    "/issues/1/comments",
                    r#"{"version":2,"body":"Reproduced"}"#
                )
                .contains("Reproduced")
            );
            assert!(
                request(address, "GET", "/issues?status=doing&q=sync", "").contains("Fix sync")
            );
            assert!(
                request(address, "POST", "/issues", r#"{"title":" "}"#).starts_with("HTTP/1.1 422")
            );
            assert!(request(address, "GET", "/issues/99", "").starts_with("HTTP/1.1 404"));
        })
        .await;
        server.abort();
        let _ = server.await;
        result.unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
}
