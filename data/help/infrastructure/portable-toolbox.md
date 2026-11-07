# Move image archives and back up service volumes

## Move prepared images with Devline

The image vault is independent of Docker's cache. Transfer exact releases with a local vault archive, then load the verified images into Docker on the receiving Mac.

```sh
devline infra export ./project-images.devlineinfra \
  --image postgres@18.6 --image redis@8.10.1
shasum -a 256 ./project-images.devlineinfra
```

On the receiving Mac, compare the checksum, import the file, then restore the image into the local Docker engine.

```sh
devline infra import ./project-images.devlineinfra
devline infra verify postgres@18.6
devline infra restore postgres@18.6
```

The vault contains service image archives. Install Docker, application runtimes, CLI tools, and project dependencies on the destination beforehand. Keep project credentials and any data volume backups separate.

## Export images with Docker

Use the local image reference recorded by Devline or shown by `docker image ls`. If it is only in the vault, run `devline infra restore postgres@18.6` first. The example adds a transport tag to a prepared PostgreSQL image. Replace `IMAGE_ID` with the exact local image ID displayed by `docker image inspect` for the prepared service.

```sh
docker image inspect "$IMAGE_ID" --format '{{.Id}} {{.Os}}/{{.Architecture}}'
docker image tag "$IMAGE_ID" devline-offline/postgres:18.6
docker image save -o devline-postgres-18.6-arm64.tar devline-offline/postgres:18.6
shasum -a 256 devline-postgres-18.6-arm64.tar
```

On the receiving Mac, compare the archive SHA-256 with the separately recorded checksum, then load it.

```sh
docker image load -i devline-postgres-18.6-arm64.tar
docker image inspect devline-offline/postgres:18.6 --format '{{.Id}} {{.Os}}/{{.Architecture}}'
```

Compare `.Id` with the catalog's image config digest. An image ID identifies the configuration of the platform image. The registry digest identifies the registry manifest. A local archive can preserve the image ID even when the engine does not retain the original registry manifest reference.

## Run Compose from cached images

Point Compose at the actual local tag you saved and loaded. Set `pull_policy: never` and use the command-line pull guard. Supply the application's built image or dependencies separately.

```yaml
services:
  postgres:
    image: devline-offline/postgres:18.6
    platform: linux/arm64
    pull_policy: never
    environment:
      POSTGRES_DB: devline
      POSTGRES_USER: devline
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:?set a local password}
    ports:
      - "127.0.0.1:5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql
volumes:
  pgdata:
```

```sh
docker compose up --detach --pull never --no-build
docker compose ps
docker compose logs --tail 50 postgres
```

Inside this Compose network, a sibling app connects to `postgres:5432`. A macOS host app uses `127.0.0.1:5432`. Keep generated passwords in a local file with restrictive permissions and exclude that file from version control.

## Back up a named data volume

Use the database's logical export when available. For a cold copy, first stop the service and its writers. Determine the volume name from its managed container before removing anything.

```sh
docker inspect "$CONTAINER" --format '{{range .Mounts}}{{println .Name .Destination}}{{end}}'
docker stop "$CONTAINER"
```

Set `VOLUME` to the intended named volume and `HELPER` to a locally prepared Linux arm64 Alpine-based image containing `tar`. The catalog's Redis Alpine image works as the helper. Prepare it while connected, run `devline infra restore redis@8.10.1` if needed, and get its local image reference from `docker image ls`. The helper has networking disabled and the source volume is read-only.

```sh
docker run --rm --pull never --network none --entrypoint tar \
  --mount "type=volume,src=$VOLUME,dst=/source,readonly" \
  "$HELPER" -C /source -cf - . > service-data.tar
shasum -a 256 service-data.tar
docker start "$CONTAINER"
```

## Restore into a fresh volume

Use the exact same image version and platform for a filesystem restore. Verify the archive's checksum and contents first. Use only an archive from a trusted source. Restore into a new empty volume and attach it to a new or stopped instance at the original mount path. Verify it before replacing the working service.

```sh
tar -tf service-data.tar | head
docker volume create devline-restored-data
docker run --rm -i --pull never --network none --entrypoint tar \
  --mount type=volume,src=devline-restored-data,dst=/target \
  "$HELPER" -C /target -xpf - < service-data.tar
```

Keep the original instance until the restored database, queue, or object store passes its application smoke test. Preserve the same RabbitMQ node name when restoring its message store. Use service-specific tools for version upgrades.

[Docker image save](https://docs.docker.com/reference/cli/docker/image/save/) · [Docker image load](https://docs.docker.com/reference/cli/docker/image/load/) · [Docker Compose service attributes](https://docs.docker.com/reference/compose-file/services/) · [Docker volume migration](https://docs.docker.com/engine/storage/volumes/#back-up-restore-or-migrate-data-volumes)
