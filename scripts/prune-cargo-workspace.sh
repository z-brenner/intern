#!/usr/bin/env bash
# usage: bash scripts/prune-cargo-workspace.sh
#
# Removes every build of the workspace's own packages, in the dev and release profiles, and the
# installers and packages the app's bundles left, from the target directory, and leaves
# third-party crates alone.
#
# CI's cargo caches (.github/workflows/ci.yml) are keyed on Cargo.lock, and when the lock changes
# they restore the newest earlier entry and save a new one after the build. Every release bumps
# the workspace's version in the lock, so cargo builds the workspace's crates again under new
# names and the old builds are never used again; without this, each entry would carry every
# earlier release's builds of them, and grow toward the repository's cache limit. The jobs run
# it after a restore that was not an exact hit. Third-party crates the new lock still pins keep
# their builds; one it no longer pins stays in the entry until the cache's key prefix changes.
#
# The bundles go too. The bundler names each installer and package after the version, so after a
# release's restore the previous release's NSIS installer and .deb could sit beside the new ones:
# the Windows smoke takes the first installer by name, which is the older one, and both jobs'
# artifacts would carry both versions.
set -euo pipefail

# Nothing was restored: nothing to prune, and `cargo clean -p` would fetch the registry index to
# resolve the workspace for no reason.
if [ ! -d "${CARGO_TARGET_DIR:-target}" ]; then
  echo "no target directory: nothing to prune"
  exit 0
fi

for profile in debug release; do
  bundle="${CARGO_TARGET_DIR:-target}/$profile/bundle"
  if [ -d "$bundle" ]; then
    echo "removing $bundle"
    rm -rf "$bundle"
  fi
done

# Inside the target directory, by a path relative to it when it is the default: Git Bash on
# Windows hands node a path it can open either way.
metadata="${CARGO_TARGET_DIR:-target}/.prune-workspace-metadata.json"
trap 'rm -f "$metadata"' EXIT
cargo metadata --no-deps --format-version 1 --locked > "$metadata"
mapfile -t packages < <(
  node -e 'for (const p of JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")).packages) console.log(p.name)' "$metadata" | tr -d '\r'
)
if [ "${#packages[@]}" -eq 0 ]; then
  echo "cargo metadata listed no workspace package" >&2
  exit 1
fi
selection=()
for package in "${packages[@]}"; do selection+=(--package "$package"); done
for profile in dev release; do
  cargo clean --locked --profile "$profile" "${selection[@]}"
done
