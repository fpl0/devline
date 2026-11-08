#!/bin/sh
set -eu
printf '%s\n' '{"tasks":["review","publish"],"ready":true}' | ruby main.rb --pretty
