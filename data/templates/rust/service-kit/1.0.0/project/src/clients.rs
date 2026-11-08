use amqprs::connection::OpenConnectionArguments;
use async_nats::{ConnectOptions, ServerAddr};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use object_store::aws::{AmazonS3, AmazonS3Builder};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, PgPool, SqlitePool};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type ClientError = Box<dyn std::error::Error + Send + Sync>;

/// Construct a lazy pool inside an active Tokio runtime.
pub fn postgres(url: &str) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect_lazy(url)
}

/// Construct a lazy pool inside an active Tokio runtime.
pub fn mysql(url: &str) -> Result<MySqlPool, sqlx::Error> {
    sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(8)
        .connect_lazy(url)
}

pub async fn sqlite(path: impl AsRef<Path>) -> Result<SqlitePool, sqlx::Error> {
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
}

pub fn redis(url: &str) -> Result<redis::Client, redis::RedisError> {
    redis::Client::open(url)
}

/// Parse a RabbitMQ URL without dialing it. Use `Connection::open` when ready.
pub fn rabbitmq(url: &str) -> Result<OpenConnectionArguments, amqprs::error::Error> {
    OpenConnectionArguments::try_from(url)
}

/// Return parsed settings for `options.connect(server).await` when ready.
pub fn nats(url: &str) -> Result<(ConnectOptions, ServerAddr), ClientError> {
    let server: ServerAddr = url.parse()?;
    let mut options = ConnectOptions::new().connection_timeout(Duration::from_secs(5));
    if let (Some(user), Some(password)) = (server.username(), server.password()) {
        // async-nats expects explicit authentication settings on ConnectOptions.
        let user = percent_encoding::percent_decode_str(user).decode_utf8()?;
        let password = percent_encoding::percent_decode_str(password).decode_utf8()?;
        options = options.user_and_password(user.into_owned(), password.into_owned());
    }
    Ok((options, server))
}

/// Require explicit credentials to avoid probing a cloud metadata service.
pub fn object_storage(
    endpoint: &str,
    region: &str,
    bucket: &str,
    key: &str,
    secret: &str,
) -> Result<AmazonS3, ClientError> {
    if endpoint.is_empty()
        || region.is_empty()
        || bucket.is_empty()
        || key.is_empty()
        || secret.is_empty()
    {
        return Err("provide an S3 endpoint, region, bucket, access key and secret".into());
    }
    install_tls();
    let address = reqwest::Url::parse(endpoint)?;
    let allow_http = address.scheme() == "http"
        && matches!(
            address.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]")
        );
    Ok(AmazonS3Builder::new()
        .with_endpoint(endpoint)
        .with_region(region)
        .with_bucket_name(bucket)
        .with_access_key_id(key)
        .with_secret_access_key(secret)
        .with_allow_http(allow_http)
        .build()?)
}

fn install_tls() {
    static PROVIDER: std::sync::Once = std::sync::Once::new();
    PROVIDER.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

pub fn http_client() -> Result<reqwest::Client, reqwest::Error> {
    install_tls();
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iss: String,
}

pub fn mint_token(key: &[u8], subject: &str) -> Result<String, ClientError> {
    if key.len() < 32 || subject.is_empty() {
        return Err("provide a subject and a signing key of at least 32 bytes".into());
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims {
        sub: subject.into(),
        exp: now + 3600,
        iss: "service-kit".into(),
    };
    Ok(jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(key),
    )?)
}

pub fn verify_token(key: &[u8], token: &str) -> Result<Claims, ClientError> {
    if key.len() < 32 {
        return Err("signing key is too short".into());
    }
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&["service-kit"]);
    validation.set_required_spec_claims(&["exp", "iss", "sub"]);
    Ok(jsonwebtoken::decode::<Claims>(token, &DecodingKey::from_secret(key), &validation)?.claims)
}
