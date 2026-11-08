use anyhow::Result;
use std::{fs, io, path::Path};

const SCRIPTS: &[(&str, &str)] = &[
    ("test-offline.sh", "test-offline.sh"),
    ("run.sh", "run.sh"),
    ("build-app.sh", "build-app.sh"),
    ("setup-offline.sh", "setup-offline.sh"),
    ("bundle-offline.sh", "bundle-offline.sh"),
    ("smoke-infra.sh", "smoke-infra.sh"),
    ("scripts/swift-local.sh", "swift-local.sh"),
    ("scripts/test-services.py", "test-services.py"),
    ("prepare-browser.mjs", "prepare-browser.mjs"),
];

// Some packages, including Go's vendor and Node's node_modules, are part of
// the native project layout. These archives have explicitly configured paths.
const PACKAGES: &[(&str, &str)] = &[
    ("python", "wheelhouse"),
    ("java", "maven-repository"),
    ("dotnet", "nuget-packages"),
    ("elixir", "vendor-tools"),
    ("node", "browser-archive"),
    ("rust", "vendor"),
];

fn move_if_present(root: &Path, from: &str, to: &str) -> Result<bool> {
    let source = root.join(from);
    match fs::symlink_metadata(&source) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
        Ok(_) => (),
    }
    let target = root.join(to);
    fs::create_dir_all(target.parent().expect("asset path has a parent"))?;
    crate::templates::rename_exclusive(&source, &target)?;
    Ok(true)
}

fn rewrite(path: &Path, replacements: &[(&str, &str)]) -> Result<()> {
    let original = match fs::read_to_string(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        result => result?,
    };
    let mut updated = original.clone();
    for (from, to) in replacements {
        updated = updated.replace(from, to);
    }
    if original != updated {
        fs::write(path, updated)?;
    }
    Ok(())
}

pub(super) fn organize(root: &Path, stack: &str) -> Result<()> {
    for file in [
        "dependency-coverage.json",
        "dependency-specs.json",
        "BROWSER.json",
        "settings-offline.xml",
    ] {
        move_if_present(root, file, &format!(".devline/{file}"))?;
    }
    for (package_stack, directory) in PACKAGES {
        if *package_stack == stack {
            move_if_present(root, directory, &format!(".devline/packages/{directory}"))?;
        }
    }

    for (from, name) in SCRIPTS {
        let to = format!(".devline/scripts/{name}");
        if !move_if_present(root, from, &to)? {
            continue;
        }
        if name.ends_with(".sh") {
            rewrite(
                &root.join(&to),
                &[
                    (
                        r#"cd "$(dirname "$0")""#,
                        r#"CDPATH='' cd "$(dirname "$0")/../..""#,
                    ),
                    (
                        r#"cd "$(dirname "$0")/..""#,
                        r#"CDPATH='' cd "$(dirname "$0")/../..""#,
                    ),
                    (
                        r#""$(dirname -- "$0")" && pwd"#,
                        r#""$(dirname -- "$0")/../.." && pwd"#,
                    ),
                    ("./setup-offline.sh", "./.devline/scripts/setup-offline.sh"),
                    (
                        "./bundle-offline.sh",
                        "./.devline/scripts/bundle-offline.sh",
                    ),
                    ("./build-app.sh", "./.devline/scripts/build-app.sh"),
                    (
                        "$project/.bundle-gems",
                        "$project/.devline/cache/ruby-runtime/gems",
                    ),
                    (
                        "$project/.bundler",
                        "$project/.devline/cache/ruby-runtime/bundler",
                    ),
                    ("Use bundle-offline.sh", "Use ./devline.sh bundle"),
                    (
                        "./scripts/swift-local.sh",
                        "./.devline/scripts/swift-local.sh",
                    ),
                    (
                        "--find-links=wheelhouse",
                        "--find-links=.devline/packages/wheelhouse",
                    ),
                    (
                        "$project/wheelhouse",
                        "$project/.devline/packages/wheelhouse",
                    ),
                    (
                        "-s settings-offline.xml",
                        "-s .devline/settings-offline.xml",
                    ),
                    ("$PWD/vendor-tools/", "$PWD/.devline/packages/vendor-tools/"),
                    (
                        "readFileSync('BROWSER.json')",
                        "readFileSync('.devline/BROWSER.json')",
                    ),
                    ("cd browser-archive", "cd .devline/packages/browser-archive"),
                    (
                        "cat browser-archive/",
                        "cat .devline/packages/browser-archive/",
                    ),
                ],
            )?;
        }
    }
    // Remove the old helper directory only when all its entries were relocated.
    let scripts = root.join("scripts");
    if scripts.is_dir() && fs::read_dir(&scripts)?.next().is_none() {
        fs::remove_dir(scripts)?;
    }

    match stack {
        "dotnet" => {
            rewrite(
                &root.join("NuGet.Config"),
                &[(r#"value="nuget-packages""#, r#"value=".devline/packages/nuget-packages""#)],
            )?;
            rewrite(
                &root.join(".devline/scripts/test-services.py"),
                &[("python3 scripts/test-services.py", "./devline.sh test-services")],
            )?;
        }
        "elixir" => {
            for source in [
                "mix.exs",
                "config/config.exs",
                "lib/application.ex",
                "lib/aws_http.ex",
                "test/health_test.exs",
                "test/dependency_kit_test.exs",
            ] {
                rewrite(
                    &root.join(source),
                    &[
                        ("DevKitPhoenix", "DevlinePhoenix"),
                        (":devkit_phoenix", ":devline_phoenix"),
                    ],
                )?;
            }
        }
        "go" => rewrite(
            &root.join("config.go"),
            &[(".devline-infra/credentials.env", ".devline/services/credentials.env")],
        )?,
        "java" if root.join(".devline/packages/maven-repository").is_dir() => rewrite(
            &root.join(".devline/scripts/test-offline.sh"),
            &[
                (
                    r#"cd "$(dirname "$0")/../..""#,
                    concat!(
                        "cd \"$(dirname \"$0\")/../..\"\n",
                        "repository=$PWD/.devline/cache/java/repository\n",
                        "[ -d \"$repository\" ] || repository=$PWD/.devline/packages/maven-repository\n",
                        "repository=${DEVLINE_MAVEN_REPOSITORY:-$repository}",
                    ),
                ),
                ("$PWD/maven-repository", "$repository"),
            ],
        )?,
        "node" => {
            let package_name = (r#""name": "devkit-app""#, r#""name": "devline-app""#);
            rewrite(
                &root.join("package.json"),
                &[
                    (r#""./test-offline.sh""#, r#""./devline.sh test""#),
                    package_name,
                ],
            )?;
            for lockfile in ["package-lock.json", "node_modules/.package-lock.json"] {
                rewrite(&root.join(lockfile), &[package_name])?;
            }
            rewrite(
                &root.join(".devline/scripts/prepare-browser.mjs"),
                &[
                    ("'browser-archive'", "'.devline/packages/browser-archive'"),
                    ("'BROWSER.json'", "'.devline/BROWSER.json'"),
                ],
            )?;
        }
        "rust" if root.join(".devline/packages/vendor").is_dir() => {
            rewrite(
                &root.join(".cargo/config.toml"),
                &[(r#"directory = "vendor""#, r#"directory = ".devline/packages/vendor""#)],
            )?;
        }
        _ => (),
    }
    let guide = root.join("README.md");
    if stack == "rust" {
        rewrite(&guide, &[("`vendor/`", "`.devline/packages/vendor/`")])?;
    }
    rewrite(
        &guide,
        &[
            (
                "Use `./devkit-test.sh` for the generated project's offline test, or `./test-offline.sh` in the starter source.",
                "Use `./devline.sh test` to test the project offline.",
            ),
            ("Devline's generated `./devkit-test.sh` runs the same checks. ", ""),
            ("`./test-offline.sh` also works directly when Cargo is on `PATH`. ", ""),
            ("./test-offline.sh", "./devline.sh test"),
            ("./devkit-test.sh", "./devline.sh test"),
            ("./run.sh", "./devline.sh run"),
            ("./build-app.sh", "./devline.sh build"),
            ("./setup-offline.sh", "./devline.sh setup"),
            ("./bundle-offline.sh", "./devline.sh bundle"),
            ("./smoke-infra.sh", "./devline.sh smoke"),
            ("./devline.sh bundle exec rake test\n", ""),
            ("python3 scripts/test-services.py", "./devline.sh test-services"),
            ("devline --offline infra run .", "./devline.sh services up"),
            ("devline infra stop .", "./devline.sh services down"),
            ("`dependency-coverage.json`", "`.devline/dependency-coverage.json`"),
            ("`BROWSER.json`", "`.devline/BROWSER.json`"),
            ("(browser-archive/SOURCES.md)", "(.devline/packages/browser-archive/SOURCES.md)"),
            ("`nuget-packages`", "`.devline/packages/nuget-packages`"),
            ("$PWD/vendor-tools/", "$PWD/.devline/packages/vendor-tools/"),
            (".devline-infra/credentials.env", ".devline/services/credentials.env"),
            ("the record size limit with networking disabled by Devline.", "the record size limit using the bundled crates."),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_starter_updates_npm_test_and_package_identity() -> Result<()> {
        let stage = crate::templates::Stage::new(&std::env::temp_dir().canonicalize()?)?;
        let manifest = stage.0.join("package.json");
        fs::write(
            &manifest,
            r#"{"name": "devkit-app","scripts":{"test":"./test-offline.sh"}}"#,
        )?;
        let lock = stage.0.join("package-lock.json");
        fs::write(
            &lock,
            r#"{"name": "devkit-app","packages":{"":{"name": "devkit-app"}}}"#,
        )?;
        let modules = stage.0.join("node_modules");
        fs::create_dir(&modules)?;
        let installed_lock = modules.join(".package-lock.json");
        fs::write(&installed_lock, r#"{"name": "devkit-app"}"#)?;
        fs::write(stage.0.join("test-offline.sh"), "exit 0\n")?;

        organize(&stage.0, "node")?;

        assert_eq!(
            fs::read_to_string(manifest)?,
            r#"{"name": "devline-app","scripts":{"test":"./devline.sh test"}}"#
        );
        assert_eq!(
            fs::read_to_string(lock)?,
            r#"{"name": "devline-app","packages":{"":{"name": "devline-app"}}}"#
        );
        assert_eq!(
            fs::read_to_string(installed_lock)?,
            r#"{"name": "devline-app"}"#
        );
        assert!(stage.0.join(".devline/scripts/test-offline.sh").is_file());
        Ok(())
    }
}
