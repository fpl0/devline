# __PROJECT_NAME__

An in-memory reading queue with Axum and Tokio. Save a title, mark it finished, and remove it using JSON. Entries reset when the app stops.

Start the generated project in Devline or with `./devline.sh start`. Open [the API index](http://127.0.0.1:7422) and try the queue.

```sh
url=http://127.0.0.1:7422
curl -i -X PUT "$url/reading/rust-book" -H 'Content-Type: application/json' \
  -d '{"title":"The Rust Programming Language","finished":false}'
curl -fsS "$url/reading"
curl -i -X PUT "$url/reading/rust-book" -H 'Content-Type: application/json' \
  -d '{"title":"The Rust Programming Language","finished":true}'
curl -i -X DELETE "$url/reading/rust-book"
```

Run `./devline.sh test` to exercise the workflow over loopback with the bundled Cargo sources. Run `./devline.sh stop` to stop the app. Change `run.url` in `.devline/config.json` to select another loopback port.
