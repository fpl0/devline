use super::*;
use flate2::{write::GzEncoder, Compression};
use std::{io::BufRead, net::TcpListener};

fn fixture() -> (Stage, Context) {
    let stage = Stage::new(&std::env::temp_dir()).unwrap();
    let catalog = stage.0.join("catalog");
    fs::create_dir(&catalog).unwrap();
    fs::write(
        catalog.join("catalog.json"),
        r#"{"schema_version":1,"docs":[],"templates":[]}"#,
    )
    .unwrap();
    let ctx = Context::new(&json!({"home":stage.0.join("library"),"catalog":catalog})).unwrap();
    (stage, ctx)
}

fn pack(version: &str) -> DocPack {
    serde_json::from_value(
        json!({"id":"sample","stack":"go","version":version,"name":"Sample",
        "path":format!("docs/sample/{version}"),"description":"Sample docs","license":"MIT",
        "source_url":"https://example.org/docs/","upstream_version":"1.0.0"}),
    )
    .unwrap()
}

fn make_archive(path: &Path, contents: &[u8], unsafe_kind: Option<&str>) {
    let gzip = GzEncoder::new(File::create(path).unwrap(), Compression::fast());
    let mut archive = tar::Builder::new(gzip);
    let mut header = tar::Header::new_ustar();
    header.set_mode(0o644);
    header.set_size(contents.len() as u64);
    header.set_path("documents.json").unwrap();
    if unsafe_kind == Some("symlink") {
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_link_name("../outside").unwrap();
    } else if unsafe_kind == Some("traversal") {
        header.as_mut_bytes()[..100].fill(0);
        header.as_mut_bytes()[..10].copy_from_slice(b"../outside");
    }
    header.set_cksum();
    archive.append(&header, contents).unwrap();
    if unsafe_kind.is_none() {
        let attribution = b"Official fixture documentation";
        let mut header = tar::Header::new_ustar();
        header.set_mode(0o644);
        header.set_size(attribution.len() as u64);
        header.set_cksum();
        archive
            .append_data(&mut header, "SOURCES.md", attribution.as_slice())
            .unwrap();
    }
    archive.into_inner().unwrap().finish().unwrap();
}

fn checked_release(ctx: &Context, pack: DocPack, bytes: &[u8], digest_matches: bool) -> Release {
    let _ = pack_dir(ctx, &pack).unwrap();
    let dir = download_dir(ctx).unwrap();
    let archive = dir.join("sample.tar.gz");
    make_archive(&archive, bytes, None);
    let meta = hash_file(&archive).unwrap();
    let digest = if digest_matches {
        meta.sha256
    } else {
        "0".repeat(64)
    };
    let release = Release {
        url: format!(
            "https://example.org/releases/{}/sample.tar.gz",
            pack.version
        ),
        pack,
        size: meta.size,
        sha256: digest,
        release_notes: "Fixture release".into(),
    };
    let partial = dir.join(format!(
        "{}@{}-{}.part",
        release.pack.id, release.pack.version, release.sha256
    ));
    fs::rename(archive, partial).unwrap();
    let saved = Feed {
        manifest: Some(Manifest {
            schema_version: 1,
            releases: vec![release.clone()],
        }),
        ..Feed::default()
    };
    atomic_json(&feed_path(ctx), &saved).unwrap();
    release
}

#[test]
fn installs_verified_archive_caches_originals_and_rolls_back() {
    let (_stage, ctx) = fixture();
    let old = pack("1.0.0");
    let prior = ctx.home.join("prior");
    fs::create_dir(&prior).unwrap();
    let docs = br#"[{"id":"api/Test","title":"Test","body":"func Test() {}","source_url":"https://example.org/docs/Test"}]"#;
    fs::write(prior.join("documents.json"), docs).unwrap();
    install_doc_pack(&ctx, &old, &prior).unwrap();
    let release = checked_release(&ctx, pack("1.0.0+devline.1"), docs, true);
    assert_eq!(status(&ctx).unwrap()["packs"][0]["state"], "available");
    let result = install(&ctx, &json!({"id":"sample"})).unwrap();
    assert_eq!(result["count"], 1);
    assert_eq!(result["active_version"], release.pack.version);
    let (_, path) = cached_pack(&ctx, "sample", Some(&release.pack.version))
        .unwrap()
        .unwrap();
    assert_eq!(fs::read(path.join("documents.json")).unwrap(), docs);
    let result = rollback(&ctx, &json!({"id":"sample"})).unwrap();
    assert_eq!(result["active_version"], old.version);
    let db = ctx.database().unwrap();
    assert_eq!(installed_versions(&db).unwrap()["sample"].len(), 2);
    assert_eq!(installed_metadata(&db).unwrap().len(), 2);
    assert_eq!(preferences(&db).unwrap()["sample"], old.version);
    fs::write(path.join("documents.json"), b"tampered").unwrap();
    assert!(cached_pack(&ctx, "sample", Some(&release.pack.version)).is_err());
}

#[test]
fn checksum_and_archive_paths_are_verified_before_install() {
    let (_stage, ctx) = fixture();
    let release = checked_release(&ctx, pack("1.0.0"), b"[]", false);
    let result = install(&ctx, &json!({"id":"sample"})).unwrap_err();
    assert!(result.to_string().contains("checksum"));
    assert!(!pack_dir(&ctx, &release.pack).unwrap().exists());
    assert!(installed_versions(&ctx.database().unwrap())
        .unwrap()
        .is_empty());
    let stage = Stage::new(&ctx.home).unwrap();
    let archive = ctx.home.join("unsafe.tar.gz");
    for kind in ["symlink", "traversal"] {
        make_archive(&archive, b"outside", Some(kind));
        assert!(extract_archive(&archive, &stage.0).is_err());
        assert!(!ctx.home.join("outside").exists());
    }
    let invalid = Release {
        url: "https://example.org/releases/latest/sample.tar.gz".into(),
        ..release
    };
    assert!(validate_manifest(&Manifest {
        schema_version: 1,
        releases: vec![invalid]
    })
    .is_err());
}

#[test]
fn waiting_for_same_release_can_be_cancelled_before_library_changes() {
    let (_stage, ctx) = fixture();
    let release = checked_release(&ctx, pack("1.0.0"), b"[]", true);
    let lock = pack_lock_file(&ctx, &release.pack).unwrap();
    FileExt::lock_exclusive(&lock).unwrap();
    let error = install_with_progress(&ctx, "sample", None, |phase, _, _| {
        assert_eq!(phase, "waiting for another download");
        bail!("Cancelled");
    })
    .unwrap_err();
    assert_eq!(error.to_string(), "Cancelled");
    assert!(installed_versions(&ctx.database().unwrap())
        .unwrap()
        .is_empty());
}

#[test]
fn resumes_actual_http_transfer_after_interruption() {
    let stage = Stage::new(&std::env::temp_dir()).unwrap();
    let partial = stage.0.join("payload.part");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        for resumed in [false, true] {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut headers = Vec::new();
            let mut line = String::new();
            let mut request = io::BufReader::new(&socket);
            while request.read_line(&mut line).unwrap() > 0 && line != "\r\n" {
                headers.push(line.to_lowercase());
                line.clear();
            }
            if resumed {
                assert!(headers.iter().any(|line| line.trim() == "range: bytes=5-"));
                socket.write_all(b"HTTP/1.1 206 Partial Content\r\nContent-Length: 5\r\nContent-Range: bytes 5-9/10\r\nConnection: close\r\n\r\n56789").unwrap();
            } else {
                socket
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\n01234",
                    )
                    .unwrap();
            }
        }
    });
    let url = format!("http://{address}/sample.tar.gz");
    assert!(download(&url, &partial, 10, "=http", &mut |_, _, _| Ok(())).is_err());
    assert_eq!(fs::read(&partial).unwrap(), b"01234");
    download(&url, &partial, 10, "=http", &mut |_, _, _| Ok(())).unwrap();
    worker.join().unwrap();
    assert_eq!(fs::read(partial).unwrap(), b"0123456789");
}
