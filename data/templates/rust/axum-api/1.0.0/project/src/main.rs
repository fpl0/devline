use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;

#[derive(Clone, Deserialize, Serialize)]
struct Entry {
    title: String,
    finished: bool,
}

type Queue = Arc<Mutex<BTreeMap<String, Entry>>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let address: SocketAddr = std::env::var("AXUM_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1:8082".into())
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Use a loopback IP and port"))?;
    if !address.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Use a loopback IP and port",
        ));
    }
    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, router()).await
}

fn router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/healthz", get(|| async { StatusCode::NO_CONTENT }))
        .route("/reading", get(list))
        .route("/reading/{id}", get(show).put(save).delete(remove))
        .layer(axum::extract::DefaultBodyLimit::max(4096))
        .with_state(Queue::default())
}

async fn index() -> Json<BTreeMap<&'static str, &'static str>> {
    Json(BTreeMap::from([
        ("app", "Reading Queue"),
        ("health", "/healthz"),
        ("list", "/reading"),
        ("entry", "/reading/{id}"),
    ]))
}

async fn list(State(queue): State<Queue>) -> Json<BTreeMap<String, Entry>> {
    Json(queue.lock().expect("reading queue lock").clone())
}

async fn show(
    State(queue): State<Queue>,
    Path(id): Path<String>,
) -> Result<Json<Entry>, StatusCode> {
    queue
        .lock()
        .expect("reading queue lock")
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn save(
    State(queue): State<Queue>,
    Path(id): Path<String>,
    Json(mut entry): Json<Entry>,
) -> Result<(StatusCode, Json<Entry>), StatusCode> {
    entry.title = entry.title.trim().to_owned();
    if id.is_empty()
        || id.len() > 40
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || entry.title.is_empty()
        || entry.title.chars().count() > 120
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let previous = queue
        .lock()
        .expect("reading queue lock")
        .insert(id, entry.clone());
    let status = if previous.is_some() {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((status, Json(entry)))
}

async fn remove(State(queue): State<Queue>, Path(id): Path<String>) -> StatusCode {
    if queue
        .lock()
        .expect("reading queue lock")
        .remove(&id)
        .is_some()
    {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    fn request(address: SocketAddr, method: &str, path: &str, body: &str) -> String {
        let mut connection = TcpStream::connect(address).unwrap();
        connection
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        write!(
            connection,
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ).unwrap();
        let mut response = String::new();
        connection.read_to_string(&mut response).unwrap();
        response
    }

    #[tokio::test]
    async fn serves_the_reading_workflow_over_loopback() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, router()).await });
        let result = tokio::task::spawn_blocking(move || {
            assert!(request(address, "GET", "/", "").contains("Reading Queue"));
            let path = "/reading/rust-book";
            assert!(
                request(address, "PUT", path, r#"{"title":" ","finished":false}"#)
                    .starts_with("HTTP/1.1 400")
            );
            assert!(
                request(
                    address,
                    "PUT",
                    path,
                    r#"{"title":"Rust Book","finished":false}"#
                )
                .starts_with("HTTP/1.1 201")
            );
            assert!(request(address, "GET", "/reading", "").contains("Rust Book"));
            assert!(
                request(
                    address,
                    "PUT",
                    path,
                    r#"{"title":"Rust Book","finished":true}"#
                )
                .starts_with("HTTP/1.1 200")
            );
            assert!(request(address, "GET", path, "").contains(r#""finished":true"#));
            assert!(request(address, "DELETE", path, "").starts_with("HTTP/1.1 204"));
            assert!(request(address, "GET", path, "").starts_with("HTTP/1.1 404"));
        })
        .await;
        server.abort();
        result.unwrap();
    }
}
