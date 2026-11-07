## Capture a message

Configure the app to use SMTP at `127.0.0.1:1025` with local capture settings. The web inbox is at `http://127.0.0.1:8025`. Replace either port if you changed it in Devline.

```sh
printf 'From: sender@example.test\r\nTo: recipient@example.test\r\nSubject: Offline probe\r\n\r\nMailpit captured this locally.\r\n' > mailpit-probe.eml
curl -fsS --url smtp://127.0.0.1:1025 --mail-from sender@example.test \
  --mail-rcpt recipient@example.test --upload-file mailpit-probe.eml
curl -fsS http://127.0.0.1:8025/readyz
curl -fsS http://127.0.0.1:8025/api/v1/messages
```

Use the web inbox to inspect the HTML, plain text, headers, and attachments. The `/api/v1/` page inside the local service includes its interactive API reference. Keep SMTP relay configuration disabled for local-only test mail.

## Keep and move a test inbox

Devline stores the SQLite database at `/data/mailpit.db` with a 5,000-message retention limit. Stop the container before copying the volume so database sidecar files remain consistent. Use the [cold volume backup](https://github.com/fpl0/devline/blob/main/scripts/infrastructure-docs/portable-toolbox.md) and restore into a fresh volume at `/data`. Verify the message count after restart.

[Mailpit Docker settings](https://mailpit.axllent.org/docs/install/docker/) · [Mailpit API](https://mailpit.axllent.org/docs/api-v1/)
