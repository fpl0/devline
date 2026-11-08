# __PROJECT_NAME__

A Ruby on Rails 8.1 starter with Puma, server-rendered ERB views, and SQLite. The home page lets you add notes and see them after a restart. Its CSS is served locally through Propshaft.

Use Ruby 4.0.9 with Bundler 4.0.20 on Apple Silicon macOS. Install Xcode Command Line Tools before going offline so Bundler can compile native gems. The checked-in `vendor/cache/` contains the gems selected by `Gemfile.lock`.

```sh
./run.sh
./test-offline.sh
```

Open [localhost:3000](http://127.0.0.1:3000) after starting the server. The development database lives at `storage/development.sqlite3`. The offline test uses a temporary database.

For other Rails commands, install the bundled gems and run Rails through Bundler.

```sh
./bundle-offline.sh install
./bundle-offline.sh exec ruby bin/rails console
```
