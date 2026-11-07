use serde::Serialize;

const MAX_TEXT_BYTES: usize = 1_048_576;

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(rename_all = "camelCase")]
struct TextStats {
    words: usize,
    lines: usize,
    unicode_scalars: usize,
    utf8_bytes: usize,
}

#[tauri::command]
fn inspect_text(text: &str) -> Result<TextStats, &'static str> {
    if text.len() > MAX_TEXT_BYTES {
        return Err("Text is limited to 1 MiB.");
    }

    Ok(TextStats {
        words: text.split_whitespace().count(),
        lines: text.lines().count(),
        unicode_scalars: text.chars().count(),
        utf8_bytes: text.len(),
    })
}

fn register_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![inspect_text])
}

fn main() {
    register_commands(tauri::Builder::default())
        .run(tauri::generate_context!())
        .expect("could not start the text inspector");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_embedded_assets_and_invokes_rust_from_the_main_window() {
        let app = register_commands(tauri::test::mock_builder())
            .build(tauri::generate_context!())
            .unwrap();
        for asset in ["index.html", "styles.css", "inspector.js"] {
            assert!(app.asset_resolver().get(asset.into()).is_some(), "{asset}");
        }

        let webview = tauri::WebviewWindowBuilder::from_config(&app, &app.config().app.windows[0])
            .unwrap()
            .build()
            .unwrap();
        let request = || tauri::webview::InvokeRequest {
            cmd: "inspect_text".into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: tauri::ipc::InvokeBody::Json(r#"{"text":"A word"}"#.parse().unwrap()),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.to_owned(),
        };
        let response = tauri::test::get_ipc_response(&webview, request())
            .unwrap()
            .deserialize::<TextStats>()
            .unwrap();
        assert_eq!(response, inspect_text("A word").unwrap());

        let other = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
            .build()
            .unwrap();
        assert!(tauri::test::get_ipc_response(&other, request()).is_err());
    }

    #[test]
    fn counts_unicode_and_mixed_line_endings() {
        assert_eq!(
            inspect_text("café\t👩‍💻\r\n次\n").unwrap(),
            TextStats {
                words: 3,
                lines: 2,
                unicode_scalars: 12,
                utf8_bytes: 23,
            }
        );
    }

    #[test]
    fn handles_empty_input_and_blank_lines() {
        assert_eq!(
            inspect_text("").unwrap(),
            TextStats {
                words: 0,
                lines: 0,
                unicode_scalars: 0,
                utf8_bytes: 0,
            }
        );
        assert_eq!(inspect_text("\n\n").unwrap().lines, 2);
    }

    #[test]
    fn limits_bytes_before_processing() {
        let text = "é".repeat(MAX_TEXT_BYTES / 2);
        assert_eq!(inspect_text(&text).unwrap().utf8_bytes, MAX_TEXT_BYTES);
        assert_eq!(
            inspect_text(&(text + "!")),
            Err("Text is limited to 1 MiB.")
        );
    }
}
