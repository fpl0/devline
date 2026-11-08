mod releases;

use axum::{
    extract::Request,
    http::{StatusCode, header, uri::Authority},
    middleware::{self, Next},
    response::Response,
};
use service_kit::{clients, notes_router};
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), clients::ClientError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let address: SocketAddr = env::var("SERVICE_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()?;
    if !address.ip().is_loopback() {
        return Err("SERVICE_LISTEN must use a loopback IP and port".into());
    }
    let path = env::var("SERVICE_DB").unwrap_or_else(|_| "notes.sqlite3".into());
    let notes = notes_router(clients::sqlite(path).await?).await?;
    let routes = if env::var_os("DEVLINE_POSTGRES_URL").is_some() {
        releases::Lab::open().await?.routes()
    } else {
        axum::Router::new().route(
            "/",
            axum::routing::get(|| async {
                axum::Json(serde_json::json!({
                    "app": "Release Rehearsal",
                    "local_notes": "/notes",
                    "health": "/healthz",
                }))
            }),
        )
    };
    let app = notes
        .merge(routes)
        .layer(middleware::from_fn(local_requests));
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(address = %listener.local_addr()?, "Service ready");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

async fn local_requests(request: Request, next: Next) -> Result<Response, StatusCode> {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<Authority>().ok())
        .ok_or(StatusCode::FORBIDDEN)?;
    let host = host.host();
    if host != "localhost"
        && !host
            .trim_matches(['[', ']'])
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
    {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(request).await)
}
