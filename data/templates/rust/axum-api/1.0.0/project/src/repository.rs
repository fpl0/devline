use crate::domain::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    schema: u32,
    next_id: u64,
    issues: BTreeMap<u64, Issue>,
}
pub struct Repository {
    path: PathBuf,
    data: Snapshot,
    _lock: File,
}
impl Repository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().to_owned();
        let dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(dir)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path.with_extension("lock"))?;
        lock.try_lock().map_err(|_| {
            Error::Storage("another Issue Desk process is using this data file".into())
        })?;
        let data = match File::open(&path) {
            Ok(f) => {
                if f.metadata()?.len() > 128 * 1024 * 1024 {
                    return Err(Error::Storage("store exceeds the 128 MiB capacity".into()));
                }
                serde_json::from_reader(BufReader::new(f))?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Snapshot {
                schema: 1,
                next_id: 1,
                issues: BTreeMap::new(),
            },
            Err(e) => return Err(e.into()),
        };
        if data.schema != 1
            || data.next_id != data.issues.len() as u64 + 1
            || data.issues.len() > 10000
        {
            return Err(Error::Storage(
                "unsupported store schema or capacity".into(),
            ));
        }
        for (id, i) in &data.issues {
            let mut input = Input {
                title: i.title.clone(),
                description: i.description.clone(),
                priority: i.priority,
                status: i.status,
                version: i.version,
            };
            if *id != i.id
                || *id == 0
                || *id >= data.next_id
                || i.version != i.history.len() as u64
                || i.history.len() > 1000
                || i.history.is_empty()
                || input.validate().is_err()
            {
                return Err(Error::Storage(
                    "invalid issue in store; restore a valid backup".into(),
                ));
            }
        }
        Ok(Self {
            path,
            data,
            _lock: lock,
        })
    }
    fn commit(&mut self, next: Snapshot) -> Result<(), Error> {
        let temp = self
            .path
            .with_extension(format!("{}.{}.tmp", std::process::id(), now()));
        let outcome = (|| {
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp)?;
            serde_json::to_writer(&mut f, &next)?;
            f.write_all(b"\n")?;
            if f.metadata()?.len() > 128 * 1024 * 1024 {
                return Err(Error::Invalid(
                    "store exceeds the 128 MiB capacity; no changes committed".into(),
                ));
            }
            f.sync_all()?;
            fs::rename(&temp, &self.path)?;
            Ok::<(), Error>(())
        })();
        let _ = fs::remove_file(&temp);
        outcome?;
        self.data = next;
        Ok(())
    }
    pub fn list(&self, status: Option<Status>, query: &str) -> Vec<Issue> {
        let q = query.to_lowercase();
        self.data
            .issues
            .values()
            .filter(|i| {
                status.is_none_or(|s| i.status == s)
                    && format!("{} {}", i.title, i.description)
                        .to_lowercase()
                        .contains(&q)
            })
            .cloned()
            .collect()
    }
    pub fn get(&self, id: u64) -> Result<Issue, Error> {
        self.data.issues.get(&id).cloned().ok_or(Error::Missing)
    }
    pub fn create(&mut self, mut input: Input) -> Result<Issue, Error> {
        input.validate()?;
        if input.status != Status::Open || input.version != 0 {
            return Err(Error::Invalid(
                "new issues must be open with no version".into(),
            ));
        }
        if self.data.issues.len() >= 10000 {
            return Err(Error::Invalid("local desk capacity is 10000 issues".into()));
        }
        let at = now();
        let issue = Issue {
            id: self.data.next_id,
            title: input.title.clone(),
            description: input.description,
            priority: input.priority,
            status: Status::Open,
            version: 1,
            created_at_ms: at,
            updated_at_ms: at,
            history: vec![Event {
                at_ms: at,
                kind: "created".into(),
                detail: input.title,
            }],
        };
        let mut next = self.data.clone();
        next.next_id += 1;
        next.issues.insert(issue.id, issue.clone());
        self.commit(next)?;
        Ok(issue)
    }
    pub fn update(&mut self, id: u64, mut input: Input) -> Result<Issue, Error> {
        input.validate()?;
        let mut issue = self.get(id)?;
        if issue.version != input.version {
            return Err(Error::Conflict);
        }
        if !allowed(issue.status, input.status) {
            return Err(Error::Invalid(
                "allowed transitions: open → doing → done → open".into(),
            ));
        }
        if issue.history.len() >= 1000 {
            return Err(Error::Invalid(
                "issue history limit reached (1000 events)".into(),
            ));
        }
        let detail = format!("{:?} → {:?}", issue.status, input.status);
        issue.title = input.title;
        issue.description = input.description;
        issue.priority = input.priority;
        issue.status = input.status;
        issue.version += 1;
        issue.updated_at_ms = now();
        issue.history.push(Event {
            at_ms: issue.updated_at_ms,
            kind: "updated".into(),
            detail,
        });
        let mut next = self.data.clone();
        next.issues.insert(id, issue.clone());
        self.commit(next)?;
        Ok(issue)
    }
    pub fn comment(&mut self, id: u64, version: u64, body: String) -> Result<Issue, Error> {
        let body = body.trim();
        if !(1..=2000).contains(&body.chars().count()) {
            return Err(Error::Invalid(
                "comment must contain 1–2000 characters".into(),
            ));
        }
        let mut issue = self.get(id)?;
        if issue.version != version {
            return Err(Error::Conflict);
        }
        if issue.history.len() >= 1000 {
            return Err(Error::Invalid(
                "issue history limit reached (1000 events)".into(),
            ));
        }
        issue.version += 1;
        issue.updated_at_ms = now();
        issue.history.push(Event {
            at_ms: issue.updated_at_ms,
            kind: "comment".into(),
            detail: body.into(),
        });
        let mut next = self.data.clone();
        next.issues.insert(id, issue.clone());
        self.commit(next)?;
        Ok(issue)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn input(title: &str) -> Input {
        Input {
            title: title.into(),
            description: String::new(),
            priority: Priority::Normal,
            status: Status::Open,
            version: 0,
        }
    }
    #[test]
    fn persistence_lock_conflict_and_workflow() {
        let dir =
            std::env::temp_dir().join(format!("issue-repository-{}-{}", std::process::id(), now()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("issues.json");
        {
            let mut repo = Repository::open(&path).unwrap();
            assert!(Repository::open(&path).is_err());
            let i = repo.create(input("Offline sync")).unwrap();
            assert_eq!(i.id, 1);
            let mut update = input("Offline sync");
            update.version = 1;
            update.status = Status::Doing;
            repo.update(1, update).unwrap();
            assert!(matches!(
                repo.comment(1, 1, "stale".into()),
                Err(Error::Conflict)
            ));
            repo.comment(1, 2, "confirmed".into()).unwrap();
        }
        {
            let repo = Repository::open(&path).unwrap();
            assert_eq!(repo.get(1).unwrap().history.len(), 3);
            assert_eq!(repo.list(Some(Status::Doing), "sync").len(), 1);
        }
        fs::write(&path, "corrupt").unwrap();
        assert!(Repository::open(&path).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn validation_and_failed_commit_do_not_publish() {
        let dir =
            std::env::temp_dir().join(format!("issue-failure-{}-{}", std::process::id(), now()));
        let path = dir.join("issues.json");
        let mut repo = Repository::open(&path).unwrap();
        assert!(repo.create(input(" ")).is_err());
        repo.create(input("Keep")).unwrap();
        repo.path = dir.clone();
        assert!(repo.create(input("must not persist")).is_err());
        assert_eq!(repo.list(None, "").len(), 1);
        drop(repo);
        fs::remove_dir_all(dir).unwrap();
    }
}
