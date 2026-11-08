use amqprs::{
    BasicProperties,
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicGetArguments, BasicPublishArguments, QueueDeclareArguments},
    connection::Connection,
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::StreamExt;
use object_store::{ObjectStoreExt, aws::AmazonS3};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use service_kit::clients::{self, ClientError};
use sqlx::{MySqlPool, PgPool};
use std::{env, future::Future, sync::Arc, time::Duration};
use tokio::time::{sleep, timeout};

const STEP_TIMEOUT: Duration = Duration::from_secs(8);
const SERVICE_ERROR: StatusCode = StatusCode::SERVICE_UNAVAILABLE;

#[derive(Deserialize, Serialize)]
struct Release {
    id: String,
    notes: String,
    complete: bool,
}

pub struct Lab {
    postgres: PgPool,
    audit: MySqlPool,
    cache: redis::Client,
    broker: String,
    events: String,
    objects: AmazonS3,
    http: reqwest::Client,
    search: String,
    search_key: String,
    mail: String,
}

impl Lab {
    pub async fn open() -> Result<Self, ClientError> {
        let lab = Self {
            postgres: clients::postgres(&setting("POSTGRES_URL")?)
                .map_err(|_| "invalid PostgreSQL settings")?,
            audit: clients::mysql(&setting("MYSQL_URL")?).map_err(|_| "invalid MySQL settings")?,
            cache: clients::redis(&setting("REDIS_URL")?).map_err(|_| "invalid Redis settings")?,
            broker: setting("RABBITMQ_URL")?,
            events: setting("NATS_URL")?,
            objects: clients::object_storage(
                &setting("SEAWEEDFS_URL")?,
                "us-east-1",
                "devline",
                "devline",
                &setting("SEAWEEDFS_PASSWORD")?,
            )
            .map_err(|_| "invalid SeaweedFS settings")?,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .redirect(reqwest::redirect::Policy::none())
                .no_proxy()
                .build()?,
            search: setting("MEILISEARCH_URL")?,
            search_key: setting("MEILISEARCH_PASSWORD")?,
            mail: format!("http://127.0.0.1:{}", setting("MAILPIT_WEB_PORT")?),
        };
        timeout(
            STEP_TIMEOUT,
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS releases (
                    id text PRIMARY KEY, notes text NOT NULL, complete boolean NOT NULL DEFAULT false
                )",
            )
            .execute(&lab.postgres),
        )
        .await?
        .map_err(|_| "could not initialize releases in PostgreSQL")?;
        timeout(
            STEP_TIMEOUT,
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS release_audit (
                    id varchar(40) PRIMARY KEY, recorded_at timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP
                )",
            )
            .execute(&lab.audit),
        )
        .await?
        .map_err(|_| "could not initialize audit in MySQL")?;
        Ok(lab)
    }

    pub fn routes(self) -> Router {
        Router::new()
            .route("/", get(index))
            .route("/releases/{id}", post(create).get(show))
            .route("/releases/{id}/artifact", get(download))
            .route("/search", get(search))
            .layer(DefaultBodyLimit::max(4096))
            .with_state(Arc::new(self))
    }

    async fn start_release(&self, release: &Release) -> Result<bool, ClientError> {
        let row =
            sqlx::query("INSERT INTO releases (id, notes) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(&release.id)
                .bind(&release.notes)
                .execute(&self.postgres)
                .await?;
        Ok(row.rows_affected() == 1)
    }

    async fn upload(&self, release: &Release) -> Result<(), ClientError> {
        self.objects
            .put(&object_path(&release.id), release.notes.clone().into())
            .await?;
        Ok(())
    }

    async fn deliver(&self, release: &Release) -> Result<(), ClientError> {
        let connection = Connection::open(&clients::rabbitmq(&self.broker)?).await?;
        connection
            .register_callback(DefaultConnectionCallback)
            .await?;
        let channel = connection.open_channel(None).await?;
        channel.register_callback(DefaultChannelCallback).await?;
        let (queue, _, _) = channel
            .queue_declare(QueueDeclareArguments::exclusive_server_named())
            .await?
            .ok_or("RabbitMQ did not declare a queue")?;
        channel
            .basic_publish(
                BasicProperties::default(),
                release.id.as_bytes().to_vec(),
                BasicPublishArguments::new("", &queue),
            )
            .await?;
        // The broker must route the published message to this isolated queue
        // before it can be fetched on the same channel.
        let (_, _, payload) = channel
            .basic_get(BasicGetArguments::new(&queue).no_ack(true).finish())
            .await?
            .ok_or("RabbitMQ did not deliver the job")?;
        channel.close().await?;
        connection.close().await?;
        if payload != release.id.as_bytes() {
            return Err("RabbitMQ delivery did not match the release".into());
        }
        Ok(())
    }

    async fn broadcast(&self, release: &Release) -> Result<(), ClientError> {
        let (options, server) = clients::nats(&self.events)?;
        let connection = options.connect(server).await?;
        let subject = format!("release.{}", release.id);
        let mut events = connection.subscribe(subject.clone()).await?;
        connection
            .publish(subject, release.id.as_bytes().to_vec().into())
            .await?;
        let message = events
            .next()
            .await
            .ok_or("NATS did not deliver the event")?;
        if message.payload.as_ref() != release.id.as_bytes() {
            return Err("NATS event did not match the release".into());
        }
        Ok(())
    }

    async fn record(&self, release: &Release) -> Result<(), ClientError> {
        sqlx::query("INSERT INTO release_audit (id) VALUES (?)")
            .bind(&release.id)
            .execute(&self.audit)
            .await?;
        Ok(())
    }

    async fn index(&self, release: &Release) -> Result<(), ClientError> {
        let response: Value = self
            .http
            .post(format!(
                "{}/indexes/releases/documents?primaryKey=id",
                self.search
            ))
            .bearer_auth(&self.search_key)
            .json(&[json!({"id": release.id, "notes": release.notes})])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let id = response["taskUid"]
            .as_u64()
            .ok_or("Meilisearch did not return an indexing task")?;
        loop {
            let task: Value = self
                .http
                .get(format!("{}/tasks/{id}", self.search))
                .bearer_auth(&self.search_key)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            match task["status"].as_str() {
                Some("succeeded") => return Ok(()),
                Some("enqueued" | "processing") => sleep(Duration::from_millis(200)).await,
                _ => return Err("Meilisearch rejected indexing".into()),
            }
        }
    }

    async fn notify(&self, release: &Release) -> Result<(), ClientError> {
        self.http
            .post(format!("{}/api/v1/send", self.mail))
            .json(&json!({
                "From": {"Email": "rehearsal@example.test"},
                "To": [{"Email": "reviewer@example.test"}],
                "Subject": format!("Release rehearsal {}", release.id),
                "Text": format!("Local rehearsal receipt for {}\n\n{}", release.id, release.notes),
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn finish_release(&self, release: &Release) -> Result<(), ClientError> {
        sqlx::query("UPDATE releases SET complete = true WHERE id = $1")
            .bind(&release.id)
            .execute(&self.postgres)
            .await?;
        Ok(())
    }

    async fn cached_release(&self, id: &str) -> Result<Option<Release>, ClientError> {
        let mut connection = self.cache.get_multiplexed_async_connection().await?;
        let cached: Option<String> = redis::cmd("GET")
            .arg(cache_key(id))
            .query_async(&mut connection)
            .await?;
        let value: Option<Release> = cached
            .map(|encoded| serde_json::from_str(&encoded))
            .transpose()?;
        Ok(value.filter(|release| release.id == id && release.complete))
    }

    async fn cache_release(&self, release: &Release) -> Result<(), ClientError> {
        let mut connection = self.cache.get_multiplexed_async_connection().await?;
        redis::cmd("SETEX")
            .arg(cache_key(&release.id))
            .arg(30)
            .arg(serde_json::to_string(release)?)
            .query_async::<()>(&mut connection)
            .await?;
        Ok(())
    }
}

fn setting(name: &str) -> Result<String, ClientError> {
    let key = format!("DEVLINE_{name}");
    match env::var(&key) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => {
            Err(format!("missing {key}. Start the generated project with ./devline.sh run").into())
        }
    }
}

fn object_path(id: &str) -> object_store::path::Path {
    format!("releases/{id}.txt").into()
}

fn cache_key(id: &str) -> String {
    format!("release:{id}")
}

fn valid_id(id: &str) -> bool {
    id.len() <= 40
        && id.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

async fn index(State(lab): State<Arc<Lab>>) -> Json<Value> {
    Json(json!({
        "app": "Release Rehearsal",
        "health": "/healthz",
        "local_notes": "/notes",
        "rehearse": "POST /releases/{id}",
        "receipt": "/releases/{id}",
        "artifact": "/releases/{id}/artifact",
        "search": "/search?q=term",
        "mail_inbox": lab.mail,
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Notes {
    notes: String,
}

async fn create(
    State(lab): State<Arc<Lab>>,
    Path(id): Path<String>,
    Json(input): Json<Notes>,
) -> Result<Response, Response> {
    let notes = input.notes.trim();
    if !valid_id(&id) || notes.is_empty() || notes.len() > 1024 || notes.contains('\0') {
        return Err((
            StatusCode::BAD_REQUEST,
            "Use a release ID of 1 to 40 lowercase letters, numbers or hyphens and notes of 1 to 1024 bytes",
        )
            .into_response());
    }
    let mut release = Release {
        id,
        notes: notes.into(),
        complete: false,
    };
    let id = &release.id;
    if !stage("PostgreSQL", id, lab.start_release(&release)).await? {
        return Err((
            StatusCode::CONFLICT,
            "That release ID already exists. Inspect its receipt or use a new ID",
        )
            .into_response());
    }
    stage("SeaweedFS", id, lab.upload(&release)).await?;
    stage("RabbitMQ", id, lab.deliver(&release)).await?;
    stage("NATS", id, lab.broadcast(&release)).await?;
    stage("MySQL", id, lab.record(&release)).await?;
    stage("Meilisearch", id, lab.index(&release)).await?;
    stage("Mailpit", id, lab.notify(&release)).await?;
    stage("PostgreSQL", id, lab.finish_release(&release)).await?;
    release.complete = true;
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/releases/{}", release.id))],
        Json(release),
    )
        .into_response())
}

async fn stage<T>(
    name: &'static str,
    id: &str,
    action: impl Future<Output = Result<T, ClientError>>,
) -> Result<T, Response> {
    match timeout(STEP_TIMEOUT, action).await {
        Ok(Ok(value)) => Ok(value),
        _ => Err((SERVICE_ERROR, Json(json!({"id": id, "failed_stage": name}))).into_response()),
    }
}

async fn show(
    State(lab): State<Arc<Lab>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    if !valid_id(&id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let cache_timeout = Duration::from_millis(500);
    if let Ok(Ok(Some(release))) = timeout(cache_timeout, lab.cached_release(&id)).await {
        return Ok(([("x-release-source", "redis")], Json(release)));
    }
    let (notes, complete) = timeout(
        STEP_TIMEOUT,
        sqlx::query_as::<_, (String, bool)>("SELECT notes, complete FROM releases WHERE id = $1")
            .bind(&id)
            .fetch_optional(&lab.postgres),
    )
    .await
    .map_err(|_| SERVICE_ERROR)?
    .map_err(|_| SERVICE_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    let release = Release {
        id,
        notes,
        complete,
    };
    if complete {
        let _ = timeout(cache_timeout, lab.cache_release(&release)).await;
    }
    Ok(([("x-release-source", "postgres")], Json(release)))
}

async fn download(
    State(lab): State<Arc<Lab>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    if !valid_id(&id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let bytes = timeout(STEP_TIMEOUT, async {
        lab.objects.get(&object_path(&id)).await?.bytes().await
    })
    .await
    .map_err(|_| SERVICE_ERROR)?
    .map_err(|error| match error {
        object_store::Error::NotFound { .. } => StatusCode::NOT_FOUND,
        _ => SERVICE_ERROR,
    })?;
    Ok((
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        bytes,
    ))
}

#[derive(Deserialize)]
struct Search {
    q: String,
}

async fn search(
    State(lab): State<Arc<Lab>>,
    Query(query): Query<Search>,
) -> Result<Json<Value>, StatusCode> {
    let term = query.q.trim();
    if term.is_empty() || term.len() > 120 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut url = reqwest::Url::parse(&format!("{}/indexes/releases/search", lab.search))
        .map_err(|_| SERVICE_ERROR)?;
    url.query_pairs_mut().append_pair("q", term);
    let mut payload: Value = lab
        .http
        .get(url)
        .bearer_auth(&lab.search_key)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|_| SERVICE_ERROR)?
        .json()
        .await
        .map_err(|_| SERVICE_ERROR)?;
    let hits = payload["hits"].as_array_mut().ok_or(SERVICE_ERROR)?;
    Ok(Json(Value::Array(std::mem::take(hits))))
}
