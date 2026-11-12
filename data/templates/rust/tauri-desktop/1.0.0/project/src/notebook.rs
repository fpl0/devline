use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub version: u64,
    pub updated_at_ms: u64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Notebook {
    schema: u32,
    next_id: u64,
    notes: Vec<Note>,
}
pub struct Store {
    file: PathBuf,
    data: Notebook,
    _lock: File,
}
const MAX_NOTEBOOK_BYTES: usize = 16 * 1024 * 1024;
const MAX_SAFE_ID: u64 = 9_007_199_254_740_991;

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn validate(title: &str, body: &str, tags: &[String]) -> Result<(), String> {
    if title.trim().is_empty() || title.chars().count() > 160 {
        return Err("Use a title of 1–160 characters.".into());
    }
    if body.len() > 262144 {
        return Err("A note may contain at most 256 KiB of text.".into());
    }
    if tags.len() > 12
        || tags
            .iter()
            .any(|t| t.trim().is_empty() || t.chars().count() > 32)
    {
        return Err("Use at most 12 tags, each with 1–32 characters.".into());
    }
    Ok(())
}
impl Store {
    pub fn open(dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let file = dir.join("notes.json");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(dir.join("notes.lock"))
            .map_err(|e| e.to_string())?;
        lock.try_lock()
            .map_err(|_| "Another Fieldnotes instance is using this notebook.".to_string())?;
        if let Ok(info) = fs::metadata(&file) {
            if info.len() > MAX_NOTEBOOK_BYTES as u64 {
                return Err("Notebook exceeds the 16 MiB capacity.".into());
            }
        }
        let data = match fs::read(&file) {
            Ok(bytes) => serde_json::from_slice::<Notebook>(&bytes)
                .map_err(|e| format!("Cannot read notebook; restore a valid backup: {e}"))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Notebook {
                schema: 1,
                next_id: 1,
                notes: vec![],
            },
            Err(e) => return Err(e.to_string()),
        };
        Self::validate_notebook(&data)?;
        Ok(Self {
            file,
            data,
            _lock: lock,
        })
    }
    fn validate_notebook(data: &Notebook) -> Result<(), String> {
        if data.schema != 1
            || (data.next_id == 0 || data.next_id > MAX_SAFE_ID)
            || data.notes.len() > 1000
        {
            return Err("Unsupported notebook schema or capacity (1000 notes).".into());
        }
        let mut ids = BTreeSet::new();
        for n in &data.notes {
            validate(&n.title, &n.body, &n.tags)?;
            if n.id == 0
                || n.id >= data.next_id
                || n.version == 0
                || n.version > MAX_SAFE_ID
                || !ids.insert(n.id)
            {
                return Err("Notebook contains invalid or duplicate note IDs.".into());
            }
        }
        Ok(())
    }
    fn commit(&mut self, next: Notebook) -> Result<(), String> {
        let encoded = serde_json::to_vec_pretty(&next).map_err(|e| e.to_string())?;
        if encoded.len() + 1 > MAX_NOTEBOOK_BYTES {
            return Err("Notebook is full (16 MiB including JSON). Export and remove notes before adding more.".into());
        }
        let temp = self
            .file
            .with_extension(format!("{}.tmp", std::process::id()));
        let result = (|| {
            let mut f = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&temp)?;
            f.write_all(&encoded)?;
            f.write_all(b"\n")?;
            f.sync_all()?;
            fs::rename(&temp, &self.file)?;
            Ok::<(), Box<dyn std::error::Error>>(())
        })();
        let _ = fs::remove_file(&temp);
        result.map_err(|e| format!("Could not save notebook: {e}"))?;
        self.data = next;
        Ok(())
    }
    pub fn list(&self) -> Vec<Note> {
        let mut notes = self.data.notes.clone();
        notes.sort_by(|a, b| {
            b.updated_at_ms
                .cmp(&a.updated_at_ms)
                .then_with(|| a.id.cmp(&b.id))
        });
        notes
    }
    pub fn save(
        &mut self,
        id: Option<u64>,
        title: String,
        body: String,
        tags: Vec<String>,
        version: u64,
    ) -> Result<Note, String> {
        let title = title.trim().to_string();
        let tags: Vec<_> = tags
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        validate(&title, &body, &tags)?;
        let mut next = self.data.clone();
        let note = if let Some(id) = id {
            let old = next
                .notes
                .iter_mut()
                .find(|n| n.id == id)
                .ok_or("This note no longer exists.")?;
            if old.version != version || version >= MAX_SAFE_ID {
                return Err("This note changed. Reload it before saving.".into());
            }
            *old = Note {
                id,
                title,
                body,
                tags,
                version: version + 1,
                updated_at_ms: now(),
            };
            old.clone()
        } else {
            if next.notes.len() >= 1000 || next.next_id >= MAX_SAFE_ID {
                return Err("Notebook is full (1000 notes). Export and remove notes first.".into());
            }
            let n = Note {
                id: next.next_id,
                title,
                body,
                tags,
                version: 1,
                updated_at_ms: now(),
            };
            next.next_id += 1;
            next.notes.push(n.clone());
            n
        };
        self.commit(next)?;
        Ok(note)
    }
    pub fn delete(&mut self, id: u64, version: u64) -> Result<(), String> {
        let note = self
            .data
            .notes
            .iter()
            .find(|n| n.id == id)
            .ok_or("This note no longer exists.")?;
        if note.version != version {
            return Err("This note changed. Reload it before deleting.".into());
        }
        let mut next = self.data.clone();
        next.notes.retain(|n| n.id != id);
        self.commit(next)
    }
    pub fn export(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.data).map_err(|e| e.to_string())
    }
    pub fn import(&mut self, text: &str) -> Result<usize, String> {
        if text.len() > MAX_NOTEBOOK_BYTES {
            return Err("An import may contain at most 16 MiB.".into());
        }
        let imported: Notebook =
            serde_json::from_str(text).map_err(|e| format!("Invalid notebook JSON: {e}"))?;
        Self::validate_notebook(&imported)?;
        if self.data.notes.len() + imported.notes.len() > 1000
            || self.data.next_id + imported.notes.len() as u64 >= MAX_SAFE_ID
        {
            return Err("Import would exceed the 1000-note capacity.".into());
        }
        let count = imported.notes.len();
        let mut next = self.data.clone();
        for mut note in imported.notes {
            note.id = next.next_id;
            next.next_id += 1;
            note.version = 1;
            note.updated_at_ms = now();
            next.notes.push(note)
        }
        self.commit(next)?;
        Ok(count)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "fieldnotes-{label}-{}-{}",
            std::process::id(),
            now()
        ))
    }
    #[test]
    fn note_lifecycle_survives_restart_and_rejects_stale_writes() {
        let dir = directory("lifecycle");
        {
            let mut s = Store::open(&dir).unwrap();
            assert!(Store::open(&dir).is_err());
            let n = s
                .save(
                    None,
                    "Idea".into(),
                    "Keep this".into(),
                    vec!["work".into(), "work".into()],
                    0,
                )
                .unwrap();
            assert_eq!(n.tags.len(), 1);
            let n = s
                .save(Some(n.id), "Edited".into(), n.body, n.tags, n.version)
                .unwrap();
            assert!(
                s.save(Some(n.id), "stale".into(), "".into(), vec![], 1)
                    .is_err()
            );
            assert_eq!(n.version, 2);
        }
        {
            let mut s = Store::open(&dir).unwrap();
            let n = s.list().remove(0);
            assert_eq!(n.title, "Edited");
            s.delete(n.id, n.version).unwrap();
            assert!(s.list().is_empty())
        }
        fs::remove_dir_all(dir).unwrap()
    }
    #[test]
    fn import_is_validated_and_preserves_existing_notes() {
        let dir = directory("import");
        let mut s = Store::open(&dir).unwrap();
        s.save(None, "Original".into(), "Text".into(), vec![], 0)
            .unwrap();
        let export = s.export().unwrap();
        assert_eq!(s.import(&export).unwrap(), 1);
        assert_eq!(s.list().len(), 2);
        assert_ne!(s.list()[0].id, s.list()[1].id);
        assert!(s.import("{}").is_err());
        assert_eq!(s.list().len(), 2);
        assert!(s.save(None, " ".into(), "".into(), vec![], 0).is_err());
        drop(s);
        fs::remove_dir_all(dir).unwrap()
    }
    #[test]
    fn corrupt_notebook_is_never_overwritten() {
        let dir = directory("corrupt");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("notes.json"), "broken").unwrap();
        assert!(Store::open(&dir).is_err());
        assert_eq!(
            fs::read_to_string(dir.join("notes.json")).unwrap(),
            "broken"
        );
        fs::remove_dir_all(dir).unwrap()
    }
    #[test]
    fn notebook_budget_preserves_restorable_exports() {
        let dir = directory("budget");
        let mut store = Store::open(&dir).unwrap();
        store
            .save(None, "Keep me".into(), "A small note".into(), vec![], 0)
            .unwrap();
        let before = store.export().unwrap();
        let mut oversized = Notebook {
            schema: 1,
            next_id: 66,
            notes: vec![],
        };
        for id in 1..66 {
            oversized.notes.push(Note {
                id,
                title: "Large note".into(),
                body: "x".repeat(262144),
                tags: vec![],
                version: 1,
                updated_at_ms: 0,
            });
        }
        assert!(store.commit(oversized).unwrap_err().contains("16 MiB"));
        assert_eq!(store.export().unwrap(), before);
        let destination = directory("restore");
        let mut restored = Store::open(&destination).unwrap();
        restored.import(&before).unwrap();
        assert_eq!(restored.list()[0].title, "Keep me");
        assert!(
            store
                .import(r#"{"schema":1,"next_id":18446744073709551615,"notes":[]}"#)
                .is_err()
        );
        drop(store);
        drop(restored);
        fs::remove_dir_all(dir).unwrap();
        fs::remove_dir_all(destination).unwrap();
    }
}
