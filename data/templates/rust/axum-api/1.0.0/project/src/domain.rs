use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Open,
    Doing,
    Done,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    pub at_ms: u64,
    pub kind: String,
    pub detail: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Issue {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub status: Status,
    pub version: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub history: Vec<Event>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub version: u64,
}
impl Input {
    pub fn validate(&mut self) -> Result<(), Error> {
        self.title = self.title.trim().into();
        self.description = self.description.trim().into();
        if !(1..=160).contains(&self.title.chars().count()) {
            return Err(Error::Invalid("title must contain 1–160 characters".into()));
        }
        if self.description.chars().count() > 8000 {
            return Err(Error::Invalid(
                "description must contain at most 8000 characters".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Debug)]
pub enum Error {
    Missing,
    Conflict,
    Invalid(String),
    Storage(String),
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Storage(e.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Storage(e.to_string())
    }
}
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
pub fn allowed(from: Status, to: Status) -> bool {
    from == to
        || matches!(
            (from, to),
            (Status::Open, Status::Doing)
                | (Status::Doing, Status::Done)
                | (Status::Done, Status::Open)
        )
}
