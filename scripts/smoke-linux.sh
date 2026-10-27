#!/usr/bin/env bash
# The Linux smoke: a package-shaped runtime checked against its inventory,
# llama-server run from it, and the parser worker driven through the release
# fixtures with it - and, when asked, the desktop app started under Xvfb and
# the whole pipeline run without a window on a real model. Nothing here needs
# a display, a desktop session or root. See docs/linux.md.
#
# Usage:
#   scripts/smoke-linux.sh [--runtime DIR] [--app PATH] [--model GGUF]
#
#   --runtime DIR  a package-shaped runtime (DIR/runtime, DIR/licenses) to
#                  check: /usr/lib/Intern once the package is installed, or a
#                  scripts/stage-linux-runtime.sh stage. Without it, what
#                  scripts/fetch-linux-assets.sh staged is staged afresh into
#                  a temporary folder.
#   --app PATH     also start this Intern - /usr/bin/intern from the package,
#                  or target/release/intern from `npm run tauri build` -
#                  under Xvfb with a throwaway home, and check that it shows
#                  its window, writes no startup error, and that killing it
#                  outright leaves none of its processes running.
#   --model GGUF   also run the real pipeline without a window: start the
#                  runtime's llama-server on GGUF with the flags the app
#                  uses, and name a fixture with intern-analyze (INTERN_ANALYZE,
#                  or built with cargo).
#
# Needs the generated fixtures (npm run fixtures) and Node; --app also needs
# xvfb-run, dbus-run-session and xwininfo (packages xvfb, dbus, x11-utils).
set -euo pipefail

usage() {
  sed -n '2,/^set -euo/p' "$0" | sed -e '$d' -e 's/^# \{0,1\}//'
  exit 2
}

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUNTIME_ROOT=""
APP=""
MODEL=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --runtime) RUNTIME_ROOT="${2:?--runtime needs a folder}"; shift 2 ;;
    --app) APP="${2:?--app needs a path}"; shift 2 ;;
    --model) MODEL="${2:?--model needs a GGUF file}"; shift 2 ;;
    *) usage ;;
  esac
done

WORK="$(mktemp -d)"
SERVER=""
cleanup() {
  if [ -n "$SERVER" ]; then
    kill "$SERVER" 2>/dev/null || true
    wait "$SERVER" 2>/dev/null || true
  fi
  rm -rf "$WORK"
}
trap cleanup EXIT

step() {
  printf '\n== %s\n' "$*"
}

# --- the runtime ------------------------------------------------------------
if [ -n "$RUNTIME_ROOT" ]; then
  RUNTIME_ROOT="$(cd "$RUNTIME_ROOT" && pwd)"
  step "Checking the runtime in $RUNTIME_ROOT against its inventory"
  node "$ROOT/scripts/linux-runtime-inventory.mjs" verify "$RUNTIME_ROOT"
else
  step "Staging the runtime from src-tauri/resources"
  "$ROOT/scripts/stage-linux-runtime.sh" "$WORK/stage"
  RUNTIME_ROOT="$WORK/stage"
fi
RUNTIME="$RUNTIME_ROOT/runtime"

step "Starting llama-server from the runtime"
# From another folder, so the CPU backends are found beside the binary and
# not in the working directory.
(cd / && "$RUNTIME/llama-server" --version) 2>&1 | tail -n 2
# Verbose enough (-lv 4) to log which CPU backend it loaded, and from where.
devices="$(cd / && "$RUNTIME/llama-server" -lv 4 --list-devices 2>&1)" || {
  printf '%s\n' "$devices" >&2
  echo "llama-server could not list its devices" >&2
  exit 1
}
if ! grep -qF "load_backend: loaded CPU backend from $RUNTIME/libggml-cpu-" <<< "$devices"; then
  printf '%s\n' "$devices" >&2
  echo "llama-server loaded no CPU backend from $RUNTIME" >&2
  exit 1
fi
grep -o 'loaded CPU backend from .*' <<< "$devices"

step "Driving the parser worker through the release fixtures"
if ! command -v tesseract > /dev/null && [ ! -x "$RUNTIME/tesseract" ]; then
  echo "warning: no tesseract on PATH (apt install tesseract-ocr); only PP-OCR can read scans" >&2
fi
node "$ROOT/scripts/smoke-worker.mjs" --worker "$RUNTIME/intern-worker" --runtime "$RUNTIME"

# --- the pipeline, headless -------------------------------------------------
if [ -n "$MODEL" ]; then
  step "Naming a fixture through the real pipeline with $(basename "$MODEL")"
  [ -f "$MODEL" ] || { echo "no model at $MODEL" >&2; exit 1; }
  if [ -z "${INTERN_ANALYZE:-}" ]; then
    (cd "$ROOT" && cargo build --release --locked -p intern-engine --bin intern-analyze)
    TARGET="${CARGO_TARGET_DIR:-target}"
    case "$TARGET" in /*) ;; *) TARGET="$ROOT/$TARGET" ;; esac
    INTERN_ANALYZE="$TARGET/release/intern-analyze"
  fi
  PORT="${PORT:-18091}"
  KEY="$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')"
  THREADS=$(( $(getconf _NPROCESSORS_ONLN) / 2 ))
  [ "$THREADS" -lt 2 ] && THREADS=2
  [ "$THREADS" -gt 12 ] && THREADS=12
  # The flags the app starts it with (crates/intern-engine/src/server.rs).
  "$RUNTIME/llama-server" --host 127.0.0.1 --port "$PORT" --api-key "$KEY" --model "$MODEL" \
    --parallel 1 --ctx-size 8192 --n-gpu-layers 0 \
    --threads "$THREADS" --threads-batch "$THREADS" --jinja --no-webui --no-mmproj \
    > "$WORK/llama-server.log" 2>&1 &
  SERVER=$!
  for _ in $(seq 1 120); do
    if curl -sf "http://127.0.0.1:$PORT/health" | grep -q '"ok"'; then break; fi
    kill -0 "$SERVER" 2>/dev/null || { cat "$WORK/llama-server.log" >&2; echo "llama-server exited" >&2; exit 1; }
    sleep 1
  done
  curl -sf "http://127.0.0.1:$PORT/health" | grep -q '"ok"' || { echo "llama-server is not healthy on port $PORT" >&2; exit 1; }
  INTERN_RUNTIME_DIR="$RUNTIME" "$INTERN_ANALYZE" --file "$ROOT/fixtures/generated/employment-agreement.pdf" \
    --worker "$RUNTIME/intern-worker" --endpoint "http://127.0.0.1:$PORT/v1/chat/completions" \
    --api-key "$KEY" --model-id intern-local > "$WORK/analysis.json"
  node -e '
    const analysis = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8"));
    if (typeof analysis.filename !== "string" || !analysis.filename.endsWith(".pdf")) {
      console.error(`intern-analyze proposed no PDF filename: ${JSON.stringify(analysis)}`);
      process.exit(1);
    }
    console.log(`Proposed "${analysis.filename}" (${analysis.status}).`);
  ' "$WORK/analysis.json"
  kill "$SERVER"
  wait "$SERVER" 2>/dev/null || true
  SERVER=""
fi

# --- the desktop app under Xvfb ---------------------------------------------
if [ -n "$APP" ]; then
  step "Starting $APP under Xvfb"
  for tool in xvfb-run dbus-run-session xwininfo; do
    command -v "$tool" > /dev/null || { echo "--app needs $tool (apt install xvfb dbus x11-utils)" >&2; exit 1; }
  done
  APP="$(cd "$(dirname "$APP")" && pwd)/$(basename "$APP")"
  [ -x "$APP" ] || { echo "no Intern at $APP" >&2; exit 1; }
  HOME_DIR="$WORK/home"
  mkdir -p "$HOME_DIR" "$WORK/run"
  chmod 0700 "$WORK/run"
  cat > "$WORK/app-smoke.sh" << 'SMOKE'
#!/usr/bin/env bash
# Runs inside Xvfb and a private D-Bus session: starts the app, waits for its
# window, checks it did not fail to start, then kills it outright and checks
# that nothing it started outlives it.
set -euo pipefail
app="$1"
startup_error="$XDG_DATA_HOME/com.intern.app/logs/startup-error.log"
"$app" > "$HOME/app.log" 2>&1 &
pid=$!
shown=""
for _ in $(seq 1 90); do
  kill -0 "$pid" 2>/dev/null || { cat "$HOME/app.log" >&2; echo "Intern exited during startup" >&2; exit 1; }
  if xwininfo -root -tree 2>/dev/null | grep -q '"Intern"'; then shown=1; break; fi
  sleep 1
done
[ -n "$shown" ] || { cat "$HOME/app.log" >&2; echo "Intern showed no window" >&2; exit 1; }
# The window is shown before initialization and again by a failed start, so
# a window alone proves nothing: a failed start writes this log.
sleep 10
if [ -f "$startup_error" ]; then
  cat "$startup_error" >&2
  echo "Intern could not start" >&2
  exit 1
fi
kill -0 "$pid" 2>/dev/null || { cat "$HOME/app.log" >&2; echo "Intern exited after starting" >&2; exit 1; }
[ -d "$XDG_DATA_HOME/com.intern.app" ] || { echo "Intern made no data folder" >&2; exit 1; }
echo "Intern started: window shown, data in \$XDG_DATA_HOME/com.intern.app, no startup error."
children="$(awk -v parent="$pid" '$4 == parent { print $1 }' /proc/[0-9]*/stat 2>/dev/null || true)"
kill -KILL "$pid"
wait "$pid" 2>/dev/null || true
for child in $children; do
  for _ in $(seq 1 50); do
    state="$(awk '{ print $3 }' "/proc/$child/stat" 2>/dev/null || true)"
    case "$state" in "" | Z | X) break ;; esac
    sleep 0.2
  done
  case "$state" in
    "" | Z | X) ;;
    *) echo "process $child outlived Intern" >&2; kill -KILL "$child" 2>/dev/null || true; exit 1 ;;
  esac
done
echo "Killing Intern left none of its $(wc -w <<< "$children") child processes running."
SMOKE
  chmod +x "$WORK/app-smoke.sh"
  # A throwaway home, so the run starts from nothing and leaves nothing; and
  # WebKitGTK's DMA-BUF renderer off, since Xvfb has no GPU to give it.
  env HOME="$HOME_DIR" XDG_CONFIG_HOME="$HOME_DIR/.config" XDG_DATA_HOME="$HOME_DIR/.local/share" \
    XDG_CACHE_HOME="$HOME_DIR/.cache" XDG_RUNTIME_DIR="$WORK/run" WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    xvfb-run --auto-servernum --server-args="-screen 0 1280x1024x24" \
    dbus-run-session -- "$WORK/app-smoke.sh" "$APP"
fi

step "Linux smoke passed"
