use axum::{Router, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8082").await?;
    axum::serve(listener, router()).await
}

fn router() -> Router {
    Router::new().route("/", get(|| async { "Axum is ready" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn serves_a_loopback_request() {
        use std::io::{Read, Write};

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, router()).await });
        let response = tokio::task::spawn_blocking(move || {
            let mut connection = std::net::TcpStream::connect(address).unwrap();
            connection
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            connection
                .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .unwrap();
            let mut response = String::new();
            connection.read_to_string(&mut response).unwrap();
            response
        })
        .await
        .unwrap();
        server.abort();
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.ends_with("Axum is ready"));
    }
}
