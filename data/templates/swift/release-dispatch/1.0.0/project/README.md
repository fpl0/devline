# Release Dispatch

Publish a release brief and hand it off to a local team. **Run Offline** starts the six configured services and publishes the sample. Open **App Logs** to find the release ID. Each run publishes a fresh release.

- **SeaweedFS** stores the JSON brief in the S3 bucket. **Redis** keeps a ten-minute copy for reads.
- **RabbitMQ** hands the brief to the notification step, which sends the email to **Mailpit**. Open Mailpit's web inbox from Current Project.
- **Meilisearch** indexes the brief for discovery, and **NATS** broadcasts its publication on `releases.published`.

```sh
./devline.sh run
./devline.sh exec swift run ReleaseDispatch publish 'CLI 2.0' 'The release is ready for review.'
./devline.sh exec swift run ReleaseDispatch search 'CLI'
./devline.sh exec swift run ReleaseDispatch read RELEASE_UUID
./devline.sh test
```

Devline supplies the loopback endpoints and credentials. Start the services with `./devline.sh services up` before using `exec` by itself. `./devline.sh test` checks formatting and validation with the services stopped. The sample workflow runs synchronously. If a service fails, the app reports the error and any steps that already completed keep their data.
