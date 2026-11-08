#!/bin/sh
set -eu
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
ruby automation.rb summarize fixtures/sales.csv --output "$scratch/summary.json"
cat "$scratch/summary.json"
