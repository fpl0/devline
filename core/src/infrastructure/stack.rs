use super::*;
use sha2::{Digest, Sha256};
use std::net::{Ipv4Addr, TcpListener, UdpSocket};

const DIRECTORY: &str = ".devline-infra";
const OWNER: &str = "io.devline.stack";

#[derive(Clone, Deserialize, Serialize)]
struct Selection {
    id: String,
    version: String,
    ports: Vec<Port>,
}

#[derive(Deserialize, Serialize)]
struct Project {
    schema_version: u32,
    project_name: String,
    nonce: String,
    services: Vec<Selection>,
}

struct Paths {
    project: PathBuf,
    directory: PathBuf,
    compose: PathBuf,
    credentials: PathBuf,
}
struct Validated {
    store: Store,
    services: Vec<Service>,
    values: BTreeMap<String, String>,
    compose: Value,
}

fn paths(input: &str) -> Result<Paths> {
    let project = Path::new(input)
        .canonicalize()
        .context("Choose an existing project directory")?;
    if !project.is_dir() {
        bail!("Choose an existing project directory");
    }
    let directory = project.join(DIRECTORY);
    Ok(Paths {
        project,
        compose: directory.join("compose.yaml"),
        credentials: directory.join("credentials.env"),
        directory,
    })
}

fn random(bytes: usize) -> Result<String> {
    let mut random = vec![0_u8; bytes];
    File::open("/dev/urandom")?.read_exact(&mut random)?;
    Ok(random.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn name(project: &Path, nonce: &str) -> Result<String> {
    let mut hash = Sha256::new();
    hash.update(
        project
            .to_str()
            .context("Project path is not UTF-8")?
            .as_bytes(),
    );
    hash.update(nonce.as_bytes());
    Ok(format!(
        "devline-{}",
        &format!("{:x}", hash.finalize())[..24]
    ))
}

fn key(service: &str, label: &str) -> String {
    format!(
        "DEVLINE_{}_{}",
        service.replace('-', "_").to_ascii_uppercase(),
        label.replace('-', "_").to_ascii_uppercase()
    )
}

fn port_key(service: &str, id: &str) -> String {
    key(service, &format!("{id}-port"))
}

fn expand(
    value: &str,
    service: &str,
    mut replacement: impl FnMut(&str, &str) -> Result<String>,
    escape_dollar: bool,
) -> Result<String> {
    let literal = |value: &str| {
        if escape_dollar {
            value.replace('$', "$$")
        } else {
            value.to_owned()
        }
    };
    let mut result = String::new();
    let mut rest = value;
    while let Some((left, token)) = rest.split_once("{{") {
        result.push_str(&literal(left));
        let (token, tail) = token
            .split_once("}}")
            .context("Unclosed infrastructure template value")?;
        let (kind, label) = token
            .split_once(':')
            .context("Invalid infrastructure template value")?;
        if !slug(label) {
            bail!("Invalid infrastructure value name");
        }
        let variable = match kind {
            "secret" => key(service, label),
            "port" => port_key(service, label),
            _ => bail!("Unknown infrastructure value type"),
        };
        result.push_str(&replacement(kind, &variable)?);
        rest = tail;
    }
    result.push_str(&literal(rest));
    Ok(result)
}

fn compose(record: &Project, services: &[Service]) -> Result<(Value, BTreeSet<String>)> {
    let mut configured = BTreeMap::new();
    let mut volumes = BTreeMap::new();
    let mut secrets = BTreeSet::new();
    for selection in &record.services {
        let service = select(services, &format!("{}@{}", selection.id, selection.version))?;
        let mut expand_value = |value: &str| {
            expand(
                value,
                &service.id,
                |kind, variable| {
                    if kind == "secret" {
                        secrets.insert(variable.to_owned());
                    } else if !selection
                        .ports
                        .iter()
                        .any(|port| port_key(&service.id, &port.id) == variable)
                    {
                        bail!("Template refers to an unknown project port");
                    }
                    Ok(format!("${{{variable}:?Missing Devline stack value}}"))
                },
                true,
            )
        };
        let ports: Vec<_> = selection
            .ports
            .iter()
            .map(|port| {
                format!(
                    "127.0.0.1:{}:{}/{}",
                    port.host_port, port.container_port, port.protocol
                )
            })
            .collect();
        let environment = service
            .env
            .iter()
            .map(|(key, value)| Ok((key, expand_value(value)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let mounts: Vec<_> = service
            .volumes
            .iter()
            .map(|volume| {
                let name = format!("{}-{}", service.id, volume.id);
                volumes.insert(
                    name.clone(),
                    json!({"labels":{"io.devline.stack":&record.nonce}}),
                );
                format!("{name}:{}", volume.target)
            })
            .collect();
        let command = service
            .command
            .iter()
            .map(|value| expand_value(value))
            .collect::<Result<Vec<_>>>()?;
        let healthcheck = service
            .healthcheck
            .command
            .iter()
            .map(|value| expand_value(value))
            .collect::<Result<Vec<_>>>()?;
        let mut value = json!({
            "image":service.alias(),"platform":service.image.platform,"pull_policy":"never",
            "ports":ports,"environment":environment,"volumes":mounts,
            "labels":{"io.devline.stack":&record.nonce},
            "healthcheck":{
                "test":healthcheck,"interval":format!("{}s",service.healthcheck.interval_seconds),
                "timeout":format!("{}s",service.healthcheck.timeout_seconds),"retries":service.healthcheck.retries,
                "start_period":format!("{}s",service.healthcheck.start_period_seconds),
            },
        });
        if !command.is_empty() {
            value["command"] = json!(command);
        }
        configured.insert(service.id, value);
    }
    Ok((
        json!({"name":record.project_name,"services":configured,"volumes":volumes,
        "networks":{"default":{"labels":{"io.devline.stack":&record.nonce}}}}),
        secrets,
    ))
}

pub(super) fn available(port: u16, protocol: &str) -> io::Result<u16> {
    let address = (Ipv4Addr::LOCALHOST, port);
    if protocol == "udp" {
        Ok(UdpSocket::bind(address)?.local_addr()?.port())
    } else {
        Ok(TcpListener::bind(address)?.local_addr()?.port())
    }
}

fn create(ctx: &Context, paths: &Paths, request: &Value) -> Result<Value> {
    if paths.project.starts_with(ctx.home.canonicalize()?) {
        bail!("Choose a project outside the Devline library");
    }
    if fs::symlink_metadata(&paths.directory).is_ok() {
        bail!("This project already has an infrastructure configuration");
    }
    let references = request["services"]
        .as_array()
        .context("Choose infrastructure services")?;
    if references.is_empty() || references.len() > 16 {
        bail!("Choose between one and sixteen services");
    }
    let mut ports = request
        .get("ports")
        .filter(|value| !value.is_null())
        .map(|value| {
            value
                .as_object()
                .cloned()
                .context("Port overrides must be an object")
        })
        .transpose()?
        .unwrap_or_default();
    let store = Store::open(ctx)?;
    let services = catalog_services(ctx, &store.index()?)?;
    let mut selected = Vec::new();
    let mut names = BTreeSet::new();
    let mut bound = BTreeSet::new();
    let mut values = BTreeMap::new();
    for reference in references {
        let service = select(
            &services,
            reference.as_str().context("Expected service@version")?,
        )?;
        if !names.insert(service.id.clone()) {
            bail!("Select only one release per service");
        }
        let mut selected_ports = service.ports;
        for port in &mut selected_ports {
            let explicit = ports
                .remove(&format!("{}:{}", service.id, port.id))
                .map(|value| {
                    let port = value
                        .as_u64()
                        .and_then(|port| u16::try_from(port).ok())
                        .filter(|port| *port > 0);
                    port.context("Port overrides must be between 1 and 65535")
                })
                .transpose()?;
            let preferred = explicit.unwrap_or(port.host_port);
            let result = available(preferred, &port.protocol)
                .ok()
                .filter(|port_number| !bound.contains(&(port.protocol.clone(), *port_number)));
            let chosen = match (explicit, result) {
                (_, Some(port)) => port,
                (Some(port), None) => bail!("Host port {port} is already in use"),
                _ => loop {
                    let candidate = available(0, &port.protocol)?;
                    if !bound.contains(&(port.protocol.clone(), candidate)) {
                        break candidate;
                    }
                },
            };
            bound.insert((port.protocol.clone(), chosen));
            port.host_port = chosen;
            values.insert(port_key(&service.id, &port.id), chosen.to_string());
        }
        selected.push(Selection {
            id: service.id,
            version: service.version,
            ports: selected_ports,
        });
    }
    if !ports.is_empty() {
        bail!("A port override refers to a service or port that was not selected");
    }
    selected.sort_by(|a, b| a.id.cmp(&b.id));
    let nonce = random(16)?;
    let record = Project {
        schema_version: 1,
        project_name: name(&paths.project, &nonce)?,
        nonce,
        services: selected,
    };
    let (compose, secrets) = compose(&record, &services)?;
    for secret in secrets {
        values.insert(secret, random(32)?);
    }
    let staging = templates::Stage::new(&paths.project)?;
    write_json_atomic(&staging.0, "project.json", &record)?;
    write_json_atomic(&staging.0, "compose.yaml", &compose)?;
    write_credentials(&staging.0.join("credentials.env"), &values)?;
    fs::write(staging.0.join(".gitignore"), "*\n")?;
    staging.publish(&paths.directory)?;
    File::open(&paths.project)?.sync_all()?;
    Ok(summary(paths, Some(&record)))
}

fn write_credentials(path: &Path, values: &BTreeMap<String, String>) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    for (key, value) in values {
        writeln!(file, "{key}={value}")?;
    }
    file.sync_all()?;
    Ok(())
}

fn load(paths: &Paths) -> Result<Option<Project>> {
    match fs::symlink_metadata(&paths.directory) {
        Ok(metadata) if metadata.is_dir() => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        _ => bail!("Invalid infrastructure project directory"),
    }
    let record: Project = read_json(&paths.directory.join("project.json"))?;
    if record.schema_version != 1
        || record.nonce.len() != 32
        || !record
            .nonce
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || record.project_name != name(&paths.project, &record.nonce)?
        || record.services.is_empty()
        || record.services.len() > 16
    {
        bail!("This infrastructure configuration does not belong to the project directory");
    }
    let mut ids = BTreeSet::new();
    for service in &record.services {
        if !slug(&service.id) || !version(&service.version) || !ids.insert(&service.id) {
            bail!("Invalid infrastructure service identity");
        }
    }
    Ok(Some(record))
}

fn summary(paths: &Paths, record: Option<&Project>) -> Value {
    json!({"path":paths.project,"project_name":record.map(|record|&record.project_name),
        "services":record.map_or(&[][..],|record|record.services.as_slice()),"configured":record.is_some()})
}

fn credentials(
    paths: &Paths,
    record: &Project,
    required: &BTreeSet<String>,
) -> Result<BTreeMap<String, String>> {
    let metadata = fs::symlink_metadata(&paths.credentials)?;
    if !metadata.is_file() || metadata.len() > 64 * 1024 {
        bail!("Invalid stack credentials file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            bail!("Set credentials.env permissions to 0600 before using this stack");
        }
    }
    let mut result = BTreeMap::new();
    let ports: BTreeMap<_, _> = record
        .services
        .iter()
        .flat_map(|service| {
            service
                .ports
                .iter()
                .map(|port| (port_key(&service.id, &port.id), port.host_port.to_string()))
        })
        .collect();
    for line in fs::read_to_string(&paths.credentials)?.lines() {
        let (key, value) = line
            .split_once('=')
            .context("Invalid stack credentials entry")?;
        if result.contains_key(key)
            || !(ports.get(key).is_some_and(|port| port == value)
                || required.contains(key) && hash(value))
        {
            bail!("Unexpected or changed stack credentials");
        }
        result.insert(key.to_owned(), value.to_owned());
    }
    if result.len() != ports.len() + required.len() {
        bail!("Stack credentials are incomplete");
    }
    Ok(result)
}

fn validated(ctx: &Context, paths: &Paths, record: &Project) -> Result<Validated> {
    let store = Store::open(ctx)?;
    let catalog = catalog_services(ctx, &store.index()?)?;
    let mut services = Vec::new();
    let mut ports = BTreeSet::new();
    for selection in &record.services {
        let service = select(&catalog, &format!("{}@{}", selection.id, selection.version))?;
        if selection.ports.len() != service.ports.len()
            || selection
                .ports
                .iter()
                .zip(&service.ports)
                .any(|(actual, expected)| {
                    actual.id != expected.id
                        || actual.container_port != expected.container_port
                        || actual.protocol != expected.protocol
                        || actual.host_port == 0
                        || !ports.insert((actual.protocol.clone(), actual.host_port))
                })
        {
            bail!("Infrastructure port configuration differs from its pinned service");
        }
        services.push(service);
    }
    let (expected, required) = compose(record, &services)?;
    if read_json::<Value>(&paths.compose)? != expected {
        bail!("The managed Compose file has changed");
    }
    let values = credentials(paths, record, &required)?;
    Ok(Validated {
        store,
        services,
        values,
        compose: expected,
    })
}

fn command<'a>(
    paths: &'a Paths,
    record: &'a Project,
    compose: &'a Path,
    credentials: &'a Path,
) -> Result<Vec<&'a str>> {
    Ok(vec![
        "compose",
        "--ansi",
        "never",
        "--project-name",
        &record.project_name,
        "--project-directory",
        paths
            .project
            .to_str()
            .context("Project path is not UTF-8")?,
        "--file",
        compose.to_str().context("Compose path is not UTF-8")?,
        "--env-file",
        credentials
            .to_str()
            .context("Credentials path is not UTF-8")?,
    ])
}

fn project_lock(paths: &Paths, job: &mut Option<&mut Job>) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(paths.directory.join("stack.lock"))?;
    if !file.metadata()?.is_file() {
        bail!("Invalid project stack lock");
    }
    loop {
        checkpoint(job)?;
        match FileExt::try_lock_exclusive(&file) {
            Ok(()) => return Ok(file),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(75))
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn owned(docker: &docker::Docker, record: &Project, kind: &str) -> Result<Vec<Value>> {
    let filter = format!("label=com.docker.compose.project={}", record.project_name);
    let mut args = vec![kind, "ls"];
    if kind == "container" {
        args.push("--all");
    }
    args.extend(["--filter", &filter, "--format", "{{json .}}"]);
    let output = docker.read(&args, Duration::from_secs(8))?;
    let mut rows = Vec::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let row: Value = serde_json::from_str(line)?;
        let resource = row["ID"]
            .as_str()
            .or_else(|| row["Name"].as_str())
            .context("Docker did not identify a project resource")?;
        let Some(labels) = docker.resource_labels(kind, resource)? else {
            continue;
        };
        if labels[OWNER] != record.nonce {
            bail!("An unrelated Docker resource uses this project name");
        }
        if kind == "container" {
            rows.push(
                json!({"ID":row["ID"],"Names":row["Names"],"State":row["State"],
                "Status":row["Status"],"Ports":row["Ports"],"Image":row["Image"]}),
            );
        } else {
            rows.push(Value::Null);
        }
    }
    Ok(rows)
}

fn expected_names(
    docker: &docker::Docker,
    record: &Project,
    services: &[Service],
    volumes: bool,
) -> Result<()> {
    let check = |kind: &str, name: &str| -> Result<()> {
        if docker
            .resource_labels(kind, name)?
            .is_some_and(|labels| labels[OWNER] != record.nonce)
        {
            bail!("An unrelated Docker {kind} already uses the managed name {name}");
        }
        Ok(())
    };
    check("network", &format!("{}_default", record.project_name))?;
    if volumes {
        for service in services {
            for volume in &service.volumes {
                check(
                    "volume",
                    &format!("{}_{}-{}", record.project_name, service.id, volume.id),
                )?;
            }
        }
    }
    Ok(())
}

fn current(paths: &Paths, record: &Project) -> Result<Value> {
    let mut value = summary(paths, Some(record));
    let docker = docker::Docker::discover();
    value["docker"] = docker::report(&docker);
    value["running"] = json!(false);
    value["healthy"] = json!(false);
    value["containers"] = json!([]);
    if let Ok(docker) = docker {
        match owned(&docker, record, "container") {
            Ok(containers) => {
                value["running"] = json!(containers
                    .iter()
                    .any(|container| container["State"] == "running"));
                value["healthy"] = json!(
                    containers.len() == record.services.len()
                        && containers
                            .iter()
                            .all(|container| container["State"] == "running"
                                && container["Status"]
                                    .as_str()
                                    .is_some_and(|status| status.contains("(healthy)")))
                );
                value["containers"] = json!(containers);
            }
            Err(error) => value["status_error"] = json!(format!("{error:#}")),
        }
    }
    Ok(value)
}

fn redact(mut text: String, values: &BTreeMap<String, String>) -> String {
    for secret in values.values().filter(|value| hash(value)) {
        text = text.replace(secret, "[redacted]");
    }
    text
}

pub(super) fn execute(ctx: &Context, request: &Value, mut job: Option<&mut Job>) -> Result<Value> {
    let path = paths(text_arg(request, "path")?)?;
    let action = text_arg(request, "command")?;
    if action == "infra.stack.create" {
        return create(ctx, &path, request);
    }
    let record = load(&path)?;
    if action == "infra.stack.config" || action == "infra.stack.status" && record.is_none() {
        return Ok(summary(&path, record.as_ref()));
    }
    let record = record.context("Configure infrastructure for this project first")?;
    if action == "infra.stack.status" {
        return current(&path, &record);
    }
    let Validated {
        store,
        services,
        values,
        compose,
    } = validated(ctx, &path, &record)?;
    if action == "infra.stack.credentials" {
        let mut connections = Vec::new();
        for service in &services {
            if let Some(value) = service.extra.get("connection").and_then(Value::as_str) {
                let resolved = expand(
                    value,
                    &service.id,
                    |_, key| values.get(key).cloned().context("Missing connection value"),
                    false,
                )?;
                connections
                    .push(json!({"id":service.id,"version":service.version,"connection":resolved}));
            }
        }
        return Ok(json!({"path":path.credentials,"credentials":values,"connections":connections}));
    }
    let _project = project_lock(&path, &mut job)?;
    let docker = docker::Docker::discover()?;
    owned(&docker, &record, "container")?;
    // Invoke Compose with a verified private snapshot, so a concurrently edited
    // project file cannot introduce a build, bind mount, or different image.
    let invocation = templates::Stage::new(&store.root)?;
    let compose_path = invocation.0.join("compose.yaml");
    let credentials_path = invocation.0.join("credentials.env");
    write_json_atomic(&invocation.0, "compose.yaml", &compose)?;
    write_credentials(&credentials_path, &values)?;
    let mut args = command(&path, &record, &compose_path, &credentials_path)?;
    match action {
        "infra.stack.up" => {
            owned(&docker, &record, "network")?;
            owned(&docker, &record, "volume")?;
            expected_names(&docker, &record, &services, true)?;
            {
                let _lock = store.lock(&mut job)?;
                let index = store.index()?;
                for service in &services {
                    let item = index.get(&service.key()).with_context(|| {
                        format!(
                            "Prepare {}@{} before starting this project",
                            service.id, service.version
                        )
                    })?;
                    restore_with(&store, service, item, &docker, &mut job)?;
                }
            }
            args.extend([
                "up",
                "--no-build",
                "--pull",
                "never",
                "--detach",
                "--wait",
                "--wait-timeout",
                "120",
            ]);
            docker.run(&args, "Starting the offline project stack", &mut job)?;
            current(&path, &record)
        }
        "infra.stack.down" => {
            owned(&docker, &record, "network")?;
            if request["delete_volumes"] == true {
                owned(&docker, &record, "volume")?;
            }
            expected_names(
                &docker,
                &record,
                &services,
                request["delete_volumes"] == true,
            )?;
            args.push("down");
            if request["delete_volumes"] == true {
                args.push("--volumes");
            }
            docker.run(&args, "Stopping this project stack", &mut job)?;
            current(&path, &record)
        }
        "infra.stack.logs" => {
            let tail = request["tail"]
                .as_u64()
                .unwrap_or(100)
                .clamp(1, 500)
                .to_string();
            args.extend(["logs", "--no-color", "--tail", &tail]);
            let mut status = summary(&path, Some(&record));
            let log = docker
                .read(&args, Duration::from_secs(10))
                .map_err(|error| anyhow::anyhow!("{}", redact(format!("{error:#}"), &values)))?;
            status["log"] = json!(redact(log, &values));
            Ok(status)
        }
        other => bail!("Unknown infrastructure project command {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_is_local_private_and_rejects_compose_edits() -> Result<()> {
        let root = templates::Stage::new(&std::env::temp_dir())?;
        let home = root.0.join("home");
        fs::create_dir(&home)?;
        fs::write(
            root.0.join("infrastructure.json"),
            include_bytes!("../../../data/infrastructure.json"),
        )?;
        let ctx = Context {
            home,
            catalog_dir: root.0.clone(),
            catalog: crate::Catalog {
                schema_version: 1,
                docs: vec![],
                templates: vec![],
            },
        };
        let project = root.0.join("project");
        fs::create_dir(&project)?;
        let paths = paths(project.to_str().unwrap())?;
        assert_eq!(
            execute(
                &ctx,
                &json!({"command":"infra.stack.status","path":project}),
                None
            )?["configured"],
            false
        );
        let service = select(&catalog_services(&ctx, &Index::new())?, "postgres")?;
        let request = json!({"command":"infra.stack.create","path":project,
            "services":[format!("{}@{}",service.id,service.version)]});
        let status = execute(&ctx, &request, None)?;
        assert_eq!(status["configured"], true);
        assert!(status.get("credentials").is_none());
        let config: Value = read_json(&paths.compose)?;
        assert_eq!(config["services"]["postgres"]["pull_policy"], "never");
        assert!(config["services"]["postgres"]["ports"][0]
            .as_str()
            .unwrap()
            .starts_with("127.0.0.1:"));
        assert!(config["services"]["postgres"].get("build").is_none());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&paths.credentials)?.permissions().mode() & 0o777,
                0o600
            );
        }
        let request = json!({"command":"infra.stack.credentials","path":project});
        let result = execute(&ctx, &request, None)?;
        assert!(hash(
            result["credentials"]["DEVLINE_POSTGRES_PASSWORD"]
                .as_str()
                .unwrap()
        ));
        assert!(result["connections"][0]["connection"]
            .as_str()
            .unwrap()
            .contains("127.0.0.1"));
        fs::write(&paths.compose, r#"{"services":{"postgres":{"build":"."}}}"#)?;
        let error = execute(&ctx, &request, None).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("managed Compose file has changed"),
            "{error:#}"
        );
        Ok(())
    }
}
