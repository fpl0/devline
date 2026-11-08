# __PROJECT_NAME__

A Ruby 4.0 task automation starter with Thor, Rake, CSV, and Minitest. It turns a local CSV or an HTTP CSV fixture into a JSON summary. Prepare Apple Silicon Ruby and Xcode Command Line Tools beforehand so Bundler can compile the bundled JSON extension locally.

Run the sample with `./run.sh`. Pass another CSV with `./devline.sh exec ruby automation.rb summarize PATH --output summary.json`.

```sh
./bundle-offline.sh install
./bundle-offline.sh exec ruby automation.rb summarize fixtures/sales.csv --output summary.json
./bundle-offline.sh exec rake test
./test-offline.sh
```

The HTTP test starts a server on loopback. The cache contains each locked gem needed by the supported Apple Silicon runtime. `Gemfile.lock` records versions and checksums.
