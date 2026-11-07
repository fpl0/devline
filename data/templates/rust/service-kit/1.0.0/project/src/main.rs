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
    let app = notes_router(clients::sqlite(path).await?).await?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(address = %listener.local_addr()?, "Service ready");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
