pub mod clients;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

pub async fn notes_router(db: SqlitePool) -> Result<Router, sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY, text TEXT NOT NULL)")
        .execute(&db)
        .await?;
    Ok(Router::new()
        .route("/healthz", get(health))
        .route("/notes", get(list_notes))
        .route("/notes", post(add_note))
        .with_state(db))
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    database: &'static str,
}

async fn health(State(db): State<SqlitePool>) -> Result<Json<Health>, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(&db)
        .await
        .map_err(database_error)?;
    Ok(Json(Health {
        status: "ok",
        database: "sqlite",
    }))
}

#[derive(Deserialize)]
struct NewNote {
    text: String,
}

async fn add_note(
    State(db): State<SqlitePool>,
    Json(note): Json<NewNote>,
) -> Result<StatusCode, StatusCode> {
    if note.text.trim().is_empty() || note.text.len() > 1024 {
        return Err(StatusCode::BAD_REQUEST);
    }
    sqlx::query("INSERT INTO notes (text) VALUES (?)")
        .bind(note.text)
        .execute(&db)
        .await
        .map_err(database_error)?;
    Ok(StatusCode::CREATED)
}

async fn list_notes(State(db): State<SqlitePool>) -> Result<Json<Vec<String>>, StatusCode> {
    let rows = sqlx::query_scalar("SELECT text FROM notes ORDER BY id DESC LIMIT 100")
        .fetch_all(&db)
        .await
        .map_err(database_error)?;
    Ok(Json(rows))
}

fn database_error(error: sqlx::Error) -> StatusCode {
    tracing::warn!(error = %error, "SQLite request failed");
    StatusCode::SERVICE_UNAVAILABLE
}
