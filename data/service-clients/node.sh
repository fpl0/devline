#!/bin/sh
set -eu
. "$(dirname "$0")/common.sh"
cd "$project"
cache="$managed/npm-cache"
if [ -n "$shelf" ] && [ -d "$shelf/npm-cache" ]; then
    # The shelf holds far more than one project needs. Resolve against a disposable
    # clone of it; the project keeps only the archives its lock names (see below).
    resolve="$TMPDIR/shelf-npm-cache"
    merge "$artifacts/npm-cache" "$resolve"
    merge "$cache" "$resolve"
    merge "$shelf/npm-cache" "$resolve"
else
    merge "$artifacts/npm-cache" "$cache"
    resolve=$cache
fi
export npm_config_cache="$resolve" npm_config_audit=false npm_config_fund=false npm_config_update_notifier=false CI=true
if [ "$offline" = true ]; then export npm_config_offline=true; else export npm_config_prefer_offline=true; fi
add_package() { npm install --save-exact --include=dev --no-audit --no-fund "$package_name@$package_version"; }
packages
if [ "$resolve" != "$cache" ]; then
    # Rebuild the project's cache from the lock: each archive by its integrity digest,
    # with the cache index entry npm looks up by download address.
    lean="$TMPDIR/lock-npm-cache"
    mkdir -p "$lean"
    "$tool" -e '
        const fs = require("node:fs"), path = require("node:path"), crypto = require("node:crypto");
        const [from, to] = process.argv.slice(1).map(root => path.join(root, "_cacache"));
        const copy = relative => {
            if (!fs.existsSync(path.join(from, relative))) return false;
            fs.mkdirSync(path.dirname(path.join(to, relative)), { recursive: true });
            fs.copyFileSync(path.join(from, relative), path.join(to, relative), fs.constants.COPYFILE_FICLONE);
            return true;
        };
        const content = integrity => {
            for (const part of String(integrity ?? "").split(/\s+/)) {
                const cut = part.indexOf("-");
                if (cut < 1) continue;
                const hex = Buffer.from(part.slice(cut + 1).split("?")[0], "base64").toString("hex");
                copy(path.join("content-v2", part.slice(0, cut), hex.slice(0, 2), hex.slice(2, 4), hex.slice(4)));
            }
        };
        const { packages = {} } = JSON.parse(fs.readFileSync("package-lock.json", "utf8"));
        for (const [location, entry] of Object.entries(packages)) {
            if (!location || entry.link || entry.inBundle) continue;
            content(entry.integrity);
            if (!/^https?:/.test(entry.resolved ?? "")) continue;
            const key = crypto.createHash("sha256").update("make-fetch-happen:request-cache:" + entry.resolved).digest("hex");
            const bucket = path.join("index-v5", key.slice(0, 2), key.slice(2, 4), key.slice(4));
            if (!copy(bucket)) continue;
            for (const line of fs.readFileSync(path.join(from, bucket), "utf8").split("\n")) {
                try { content(JSON.parse(line.slice(line.indexOf("\t") + 1)).integrity); } catch {}
            }
        }
    ' "$resolve" "$lean"
    rm -rf "$cache"
    mv "$lean" "$cache"
    export npm_config_cache="$cache"
fi
exec /bin/sh "$catalog/project-adapters/node.sh" prepare "$project" "$managed" "$tool"
