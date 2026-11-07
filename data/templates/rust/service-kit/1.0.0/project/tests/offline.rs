use axum::body::Body;
use axum::http::{Request, StatusCode};
use service_kit::{clients, notes_router};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

#[tokio::test]
async fn client_configuration_is_local_until_used() {
    let postgres = clients::postgres("postgresql://test:test@127.0.0.1:5432/devline").unwrap();
    let mysql = clients::mysql("mysql://test:test@127.0.0.1:3306/devline").unwrap();
    assert_eq!(postgres.size(), 0);
    assert_eq!(mysql.size(), 0);
    assert!(clients::redis("redis://:test@127.0.0.1:6379/").is_ok());
    assert!(clients::rabbitmq("amqp://test:test@127.0.0.1:5672/%2f").is_ok());
    assert!(clients::nats("nats://127.0.0.1:4222").is_ok());
    assert!(
        clients::object_storage(
            "http://127.0.0.1:8333",
            "us-east-1",
            "notes",
            "local",
            "local-secret"
        )
        .is_ok()
    );
    assert!(clients::nats("not-a://valid-nats-endpoint").is_err());
}

#[test]
fn signed_token_round_trip_rejects_another_key() {
    let key = b"0123456789abcdefghijklmnopqrstuv";
    let token = clients::mint_token(key, "developer").unwrap();
    assert_eq!(clients::verify_token(key, &token).unwrap().sub, "developer");
    assert!(clients::verify_token(b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", &token).is_err());
}

#[tokio::test]
async fn notes_flow_runs_on_sqlite_and_http() {
    let db = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    let app = notes_router(db).await.unwrap();
    let response = app
        .clone()
        .oneshot(
            Request::post("/notes")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"text":"works offline"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/notes", listener.local_addr().unwrap());
    let server = tokio::spawn(async { axum::serve(listener, app).await.unwrap() });
    let notes: Vec<String> = clients::http_client()
        .unwrap()
        .get(url)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(notes, ["works offline"]);
    server.abort();
}
