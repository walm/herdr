#!/usr/bin/env bash
# dev-run.sh — run a local dev build of herdr fully isolated from any
# already-running (stable) herdr server on this machine.
#
# Isolation comes from two env overrides so the dev instance never touches
# your real session:
#   HERDR_SOCKET_PATH  -> its own server socket (client socket is auto-derived
#                         as herdr-client.sock next to it)
#   HERDR_CONFIG_PATH  -> .local/dev-config.toml (git-ignored; edit freely)
#
# Usage:
#   ./dev-run.sh                        # launch the dev TUI
#   ./dev-run.sh server reload-config   # hot-reload the dev server's config
#   ./dev-run.sh server stop            # stop the dev server
#   ./dev-run.sh server status          # check the dev server
#   DEV_RELEASE=1 ./dev-run.sh          # use the release profile (smoother TUI)
#
# Override locations if you like:
#   HERDR_DEV_RUN_DIR   socket dir      (default: /tmp/herdr-dev-$USER)
#   HERDR_DEV_CONFIG    config file     (default: <repo>/.local/dev-config.toml)
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

# Dedicated runtime dir for the dev sockets. Kept short for the AF_UNIX path
# length limit (~104 bytes on macOS).
run_dir="${HERDR_DEV_RUN_DIR:-/tmp/herdr-dev-$USER}"
mkdir -p "$run_dir"

# Git-ignored dev config; seed a starter one on first run.
config_path="${HERDR_DEV_CONFIG:-$repo_root/.local/dev-config.toml}"
if [[ ! -f "$config_path" ]]; then
  mkdir -p "$(dirname "$config_path")"
  cat >"$config_path" <<'EOF'
# Local dev config for dev-run.sh — git-ignored, edit freely.
# Hot-reload without restarting the dev server:
#   ./dev-run.sh server reload-config

[ui]
tab_number_prefix = true
EOF
  echo "dev-run: created starter dev config at $config_path" >&2
fi

profile_args=()
if [[ "${DEV_RELEASE:-0}" == "1" ]]; then
  profile_args=(--release)
fi

echo "dev-run: socket $run_dir/herdr.sock | config $config_path" >&2

# `env -u ...` drops any socket overrides inherited from a parent herdr session
# before we set our own, so this never attaches to the stable server.
exec env \
  -u HERDR_SOCKET_PATH \
  -u HERDR_CLIENT_SOCKET_PATH \
  HERDR_SOCKET_PATH="$run_dir/herdr.sock" \
  HERDR_CONFIG_PATH="$config_path" \
  cargo run ${profile_args[@]+"${profile_args[@]}"} -- "$@"
