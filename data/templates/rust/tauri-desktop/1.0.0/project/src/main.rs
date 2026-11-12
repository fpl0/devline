mod notebook;
use notebook::{Note, Store};
use std::sync::Mutex;
use tauri::{Manager, State};
type NotebookState = Mutex<Store>;
#[tauri::command]
fn list_notes(store: State<'_, NotebookState>) -> Result<Vec<Note>, String> {
    Ok(store
        .lock()
        .map_err(|_| "Notebook lock unavailable")?
        .list())
}
#[tauri::command]
fn save_note(
    store: State<'_, NotebookState>,
    id: Option<u64>,
    title: String,
    body: String,
    tags: Vec<String>,
    version: u64,
) -> Result<Note, String> {
    store
        .lock()
        .map_err(|_| "Notebook lock unavailable")?
        .save(id, title, body, tags, version)
}
#[tauri::command]
fn delete_note(store: State<'_, NotebookState>, id: u64, version: u64) -> Result<(), String> {
    store
        .lock()
        .map_err(|_| "Notebook lock unavailable")?
        .delete(id, version)
}
#[tauri::command]
fn export_notebook(store: State<'_, NotebookState>) -> Result<String, String> {
    store
        .lock()
        .map_err(|_| "Notebook lock unavailable")?
        .export()
}
#[tauri::command]
fn import_notebook(store: State<'_, NotebookState>, text: String) -> Result<usize, String> {
    store
        .lock()
        .map_err(|_| "Notebook lock unavailable")?
        .import(&text)
}
fn register<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        list_notes,
        save_note,
        delete_note,
        export_notebook,
        import_notebook
    ])
}
fn main() {
    register(tauri::Builder::default())
        .setup(|app| {
            let directory = match std::env::var_os("FIELDNOTES_DATA_DIR") {
                Some(path) => std::path::PathBuf::from(path),
                None => app.path().app_data_dir()?,
            };
            let store = Store::open(&directory).map_err(std::io::Error::other)?;
            app.manage(Mutex::new(store));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("could not start Fieldnotes");
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_ui_and_scoped_ipc() {
        let dir = std::env::temp_dir().join(format!("fieldnotes-ipc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let app = register(tauri::test::mock_builder())
            .manage(Mutex::new(Store::open(&dir).unwrap()))
            .build(tauri::generate_context!())
            .unwrap();
        for asset in ["index.html", "styles.css", "inspector.js"] {
            assert!(app.asset_resolver().get(asset.into()).is_some())
        }
        let window = tauri::WebviewWindowBuilder::from_config(&app, &app.config().app.windows[0])
            .unwrap()
            .build()
            .unwrap();
        let request = || tauri::webview::InvokeRequest {
            cmd: "list_notes".into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: tauri::ipc::InvokeBody::Json("{}".parse().unwrap()),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_owned(),
        };
        let notes = tauri::test::get_ipc_response(&window, request())
            .unwrap()
            .deserialize::<Vec<Note>>()
            .unwrap();
        assert!(notes.is_empty());
        let other = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
            .build()
            .unwrap();
        assert!(tauri::test::get_ipc_response(&other, request()).is_err());
        drop(other);
        drop(window);
        drop(app);
        let _ = std::fs::remove_dir_all(dir);
    }
}
