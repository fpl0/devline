//! The generated helper forwards app arguments without treating them as shell code.
use std::{fs, os::unix::fs::PermissionsExt, process::Command};

#[test]
fn app_arguments_use_the_run_helper_and_preserve_boundaries() {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path().join("project with spaces");
    let scripts = root.join(".devline/scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(
        root.join("devline.sh"),
        include_bytes!("../src/project_layout/devline.sh"),
    )
    .unwrap();
    fs::write(
        scripts.join("lifecycle.sh"),
        include_bytes!("../src/project_layout/lifecycle.sh"),
    )
    .unwrap();
    fs::write(scripts.join("run.sh"), "#!/bin/sh\nexit 0\n").unwrap();
    let cli = scratch.path().join("fake devline");
    fs::write(&cli, "#!/bin/sh\nprintf '<%s>\\n' \"$@\"\nexit 37\n").unwrap();
    fs::set_permissions(&cli, fs::Permissions::from_mode(0o755)).unwrap();
    let launch = |args: &[&str]| {
        Command::new("/bin/sh")
            .arg(root.join("devline.sh"))
            .args(args)
            .current_dir(scratch.path())
            .env("DEVLINE_BIN", &cli)
            .output()
            .unwrap()
    };
    let prefix = format!(
        "<--offline>\n<project>\n<--path>\n<{}>\n<run>\n",
        root.canonicalize().unwrap().display()
    );
    let args = [
        "run",
        "report",
        "two words",
        "$(touch should-not-exist)",
        "",
        "--json",
    ];
    let result = launch(&args);
    assert_eq!(result.status.code(), Some(37));
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        format!("{prefix}<-->\n</bin/sh>\n<.devline/scripts/run.sh>\n<report>\n<two words>\n<$(touch should-not-exist)>\n<>\n<--json>\n")
    );
    assert!(!root.join("should-not-exist").exists());
    assert_eq!(String::from_utf8(launch(&["run"]).stdout).unwrap(), prefix);
    assert_eq!(
        String::from_utf8(launch(&["run", "--", "--help"]).stdout).unwrap(),
        format!("{prefix}<-->\n</bin/sh>\n<.devline/scripts/run.sh>\n<--help>\n")
    );
    fs::rename(scripts.join("run.sh"), root.join("run.sh")).unwrap();
    assert!(String::from_utf8(launch(&["run", "help"]).stdout)
        .unwrap()
        .contains("<./run.sh>\n<help>"));
    fs::remove_file(root.join("run.sh")).unwrap();
    let missing = launch(&["run", "help"]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8(missing.stderr)
        .unwrap()
        .contains("no run helper"));
}
