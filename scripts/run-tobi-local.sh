#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/run-tobi-local.sh tui|serial [extra tobi args...]

Runs TOBI locally against the sample catalog in mock mode.

Examples:
  scripts/run-tobi-local.sh tui
  scripts/run-tobi-local.sh serial
  scripts/run-tobi-local.sh tui --test-proxy-setup

Environment:
  TOBI_RUN_MODE=mock|live            Defaults to mock.
  TOBI_MANIFEST=path-or-url          Defaults to tobi/sample/catalog.json.
  TOBI_DOCKER_IMAGE=image:tag        Defaults to the local x86_64 builder image.
USAGE
}

if [[ $# -lt 1 || "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

LOCAL_UI="$1"
shift

case "$LOCAL_UI" in
  tui|serial) ;;
  *)
    usage >&2
    exit 2
    ;;
esac

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="${TOBI_DOCKER_IMAGE:-tobi-ubuntu-x86_64-tobi-cross-builder:22.04}"
CACHE_DIR="${TOBI_LOCAL_CACHE_DIR:-$REPO_ROOT/out/.cache/local-tobi-run}"
RUN_MODE="${TOBI_RUN_MODE:-mock}"
MANIFEST="${TOBI_MANIFEST:-sample/catalog.json}"

if [[ "$LOCAL_UI" == "tui" && ( ! -t 0 || ! -t 1 ) ]]; then
  echo "The TUI mode needs an interactive terminal. Run this from a real terminal." >&2
  exit 1
fi

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  docker build \
    -f "$REPO_ROOT/docker/ubuntu-x86_64-tobi-cross-builder/Dockerfile" \
    -t "$IMAGE" \
    "$REPO_ROOT"
fi

mkdir -p "$CACHE_DIR/cargo" "$CACHE_DIR/target"

docker_tty_args=(-i)
if [[ -t 0 && -t 1 ]]; then
  docker_tty_args=(-it)
fi

app_args=(--mode "$RUN_MODE" --manifest "$MANIFEST")
if [[ "$LOCAL_UI" == "serial" ]]; then
  app_args+=(--serial-ui)
fi
app_args+=("$@")

exec docker run --rm "${docker_tty_args[@]}" \
  --user "$(id -u):$(id -g)" \
  -e CARGO_HOME=/cargo \
  -e CARGO_TARGET_DIR=/target \
  -e TERM="${TERM:-xterm-256color}" \
  -v "$REPO_ROOT/tobi:/workspace:ro" \
  -v "$CACHE_DIR/cargo:/cargo" \
  -v "$CACHE_DIR/target:/target" \
  -w /workspace \
  "$IMAGE" \
  cargo run --quiet -- "${app_args[@]}"
