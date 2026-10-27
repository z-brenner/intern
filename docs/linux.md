# Intern on Linux

Intern builds, runs and is tested on **Ubuntu 24.04, x86-64**. CI builds the
`.deb` from the pinned runtime on every push, installs it with apt, and starts
the installed app under Xvfb (the **Linux desktop package and smoke** job). No
Linux build is published as a release yet, and Linux builds do not update
themselves; the Windows installer remains the shipped product.

## Install a package

Take `Intern_<version>_amd64.deb` from a CI run's `intern-linux-…` artifact,
or build one (below), and install it with apt, which resolves what it depends
on - WebKitGTK 4.1, the AppIndicator library, `tesseract-ocr`, `libgomp1`,
and OpenSSL 3:

```sh
sudo apt install ./Intern_0.1.0-alpha.12_amd64.deb
intern            # or Intern from the applications menu
```

First launch is the same as on Windows: the setup screen downloads the pinned
model, or **Choose existing model files** points at one you have.

| What | Where |
| --- | --- |
| The app | `/usr/bin/intern`, and **Intern** in the applications menu |
| llama-server, the parser worker, PDFium, ONNX Runtime, the OCR models, tessdata | `/usr/lib/Intern/runtime/`, listed with every file's SHA-256 in `runtime-inventory.json` there |
| Third-party licenses | `/usr/lib/Intern/licenses/` and `/usr/lib/Intern/resources/THIRD_PARTY_NOTICES.md` |
| The model, the queue, settings, logs | `~/.local/share/com.intern.app/` (`logs/startup-error.log` if Intern cannot start, `logs/intern.log` for a crash) |
| **Start Intern when you sign in** | `~/.config/autostart/Intern.desktop` |
| The hosted model's API key | The Secret Service (GNOME Keyring, KWallet), as **Intern** |

`sudo apt remove intern` takes the app away and leaves your documents and
`~/.local/share/com.intern.app` alone.

## Build from source

The pinned toolchains are the repository's: Rust 1.88.0 and Node 24.15.0
(`.nvmrc`). On Ubuntu 24.04:

```sh
sudo apt-get install -y build-essential curl git unzip pkg-config \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf \
  tesseract-ocr
npm ci
cargo build --locked -p intern-worker --release --features windows-native
scripts/fetch-linux-assets.sh ~/.cache/intern-linux-assets
npm run tauri build -- --bundles deb -- --locked
# target/release/bundle/deb/Intern_<version>_amd64.deb
```

(`windows-native` is the feature set that builds PDFium, Tesseract and PP-OCR
into the worker; its name predates Linux packages.)

`scripts/fetch-linux-assets.sh` is the counterpart of
`scripts/fetch-windows-assets.ps1`, with the same discipline: every download
is refused unless its exact size and SHA-256 match its pin, nothing in
`src-tauri/resources` is touched until everything has been fetched and
checked, and the result is recorded file by file in
`src-tauri/resources/runtime/runtime-inventory.json` with license texts in
`src-tauri/resources/licenses/`. The cache folder keeps downloads between
runs, and they are verified again on reuse.

`npm run tauri dev` runs against the same staged runtime: Tauri copies it to
`target/debug/runtime`. To run against a runtime somewhere else, point
`INTERN_RUNTIME_DIR` at it (an absolute path):

```sh
scripts/stage-linux-runtime.sh /tmp/intern-stage        # a verified copy, shaped as installed
INTERN_RUNTIME_DIR=/tmp/intern-stage/runtime npm run tauri dev
```

### The runtime and its pins

| Part | Source | Pinned by |
| --- | --- | --- |
| `llama-server`, `libllama*`, `libggml*`, `libmtmd`, and a `libggml-cpu-*.so` per x86-64 generation | The [PrismML llama.cpp fork](https://github.com/PrismML-Eng/llama.cpp), which reads the Bonsai 2 ternary models: its prebuilt `llama-<release>-bin-ubuntu-x64.tar.gz` | The release, at the top of `scripts/fetch-linux-assets.sh`, and its archive's size and SHA-256 |
| `libpdfium.so` | pdfium-binaries `chromium/7881`, as on Windows | Size and SHA-256 |
| `libonnxruntime.so`, `ocr-models/` | ONNX Runtime 1.30.0's PyPI wheel; the PP-OCR models | `scripts/fetch-ocr-runtime.sh` and `runtime-assets.json` |
| `tessdata/eng.traineddata`, `osd.traineddata` | `tessdata_fast`, as on Windows | `runtime-assets.json` |
| `intern-worker` | This repository | Its digest, in the inventory |

**llama.cpp** is Prism's own prebuilt Ubuntu x86-64 release, CPU-only and
portable: it carries one CPU backend per x86-64 generation, from `sse42` up to
`sapphirerapids` and `zen4`, and llama-server loads at run time the one the
machine it runs on can execute. Only llama-server is staged, with the
libraries it needs and those backends - not the archive's other tools, nor its
RPC backend - and every library carries an `$ORIGIN` run path, so the runtime
folder works wherever it is put; the script refuses an archive where that is
not so, or where any library resolves to nothing. The archive is linked
against the system's OpenSSL 3, `libgomp` and `libstdc++`, which the package
depends on.

The release is one pin: `LLAMA_CPP_RELEASE` at the top of
`scripts/fetch-linux-assets.sh`, `prism-b10826-c9c34bc` today - the build the
77-document evaluation ran both shipped models on, and the one the Windows
package carries too. The table under
it holds the archive size and SHA-256 of every release that may be staged -
`prism-b10777-e3de715` and `prism-b10826-c9c34bc` - so moving to one already
there is a one-word change, and to a new one a new row from its release page.
To stage another pinned release without changing the default, set
`LLAMA_CPP_RELEASE` in the environment; the inventory records the release,
archive URL, size and SHA-256 that were staged. There is no build from source:
Prism publishes the archive with each release.

**Tesseract** is the distribution's `tesseract-ocr` (5.3.4 on Ubuntu 24.04),
a dependency of the package, rather than a build carried inside it. On Linux
it is only the fallback for when PP-OCR cannot load, and bundling it would mean
carrying and patching its whole image-library closure (Leptonica, libpng,
libjpeg, libtiff, libwebp, OpenJPEG) beside the distribution's copies. It
reads the runtime's pinned `tessdata`, the same files the Windows package
ships; the worker uses a `tesseract` in the runtime folder if one is put
there, and otherwise the one on `PATH`. OCR output can differ by a character
from the Windows package's Tesseract 5.5.2.

**`llama-server` and the worker live in `/usr/lib/Intern/runtime`, not
`/usr/bin`.** A package's `/usr/bin/llama-server` and `/usr/bin/tesseract`
would collide with the distribution's own, and llama-server looks for its CPU
backends beside itself. The app starts both from the runtime folder.

## Testing without a desktop

Everything below runs on a machine with no display and no desktop session,
such as the VM a coding agent works in. Run `npm run fixtures` first.

`scripts/smoke-linux.sh` checks a runtime against its inventory, starts its
llama-server and checks a CPU backend loads, and drives the parser worker
through the release fixtures - native text, PDFium-rendered scans through
PP-OCR, DOCX, images, and the documents it must refuse
(`scripts/smoke-worker.mjs`, the port of `scripts/smoke-worker.ps1`):

```sh
scripts/smoke-linux.sh                              # what fetch-linux-assets.sh staged
scripts/smoke-linux.sh --runtime /usr/lib/Intern    # an installed package
```

`--app PATH` also starts the real app under Xvfb, in a throwaway home and a
private D-Bus session, and checks that it shows its window, writes no startup
error, and that killing it outright leaves none of its processes running.
`--model GGUF` also runs the whole pipeline without a window: the runtime's
llama-server on that model, with the flags the app uses, and `intern-analyze`
naming a fixture through it.

```sh
sudo apt-get install -y xvfb dbus x11-utils
scripts/smoke-linux.sh --runtime /usr/lib/Intern --app /usr/bin/intern
scripts/smoke-linux.sh --app target/release/intern --model ~/models/model.gguf
```

`target/release/intern` must come from `npm run tauri build`, which builds
the frontend into it; a plain `cargo build` binary looks for the Vite dev
server instead.

To use the app by hand on a headless machine, run it the way the smoke does,
and drive or screenshot it with any X11 tool against that display:

```sh
xvfb-run -a dbus-run-session -- env WEBKIT_DISABLE_DMABUF_RENDERER=1 intern
```

`WEBKIT_DISABLE_DMABUF_RENDERER=1` keeps WebKitGTK from trying a GPU renderer
Xvfb does not have, which otherwise leaves the window blank. The D-Bus session
is what single-instance and the Secret Service use; without one Intern still
runs, as more than one copy, with no hosted-model key.

For the pipeline alone, `intern-analyze` is the same code path as the app, and
`scripts/run-internbench.sh` runs InternBench live against a llama-server -
both take the staged runtime directly:

```sh
RUNTIME=/usr/lib/Intern/runtime   # or a stage-linux-runtime.sh stage's runtime/
"$RUNTIME/llama-server" --host 127.0.0.1 --port 8080 --api-key KEY --model model.gguf \
  --ctx-size 8192 --jinja --no-webui --no-mmproj &
INTERN_RUNTIME_DIR="$RUNTIME" cargo run --locked --release -p intern-engine --bin intern-analyze -- \
  --file contract.pdf --worker "$RUNTIME/intern-worker" \
  --endpoint http://127.0.0.1:8080/v1/chat/completions --api-key KEY
MODEL=model.gguf LLAMA_SERVER="$RUNTIME/llama-server" INTERN_RUNTIME_DIR="$RUNTIME" \
  INTERN_WORKER="$RUNTIME/intern-worker" scripts/run-internbench.sh
```

## How the desktop parts behave on Linux

- **Background mode and the tray.** The tray icon uses AppIndicator, which
  Ubuntu's desktop shows; closing the window with **Run in the background** on
  keeps Intern running there, as on Windows. AppIndicator reports no clicks on
  the icon itself, so the window comes back from the icon's menu (**Open
  Intern**) rather than a left click. A desktop with no tray host (stock GNOME
  without the AppIndicator extension, a bare window manager) shows no icon:
  Intern carries on without one, and launching Intern again brings the window
  back.
- **Start Intern when you sign in** writes an XDG autostart entry,
  `~/.config/autostart/Intern.desktop`, which starts it with `--minimized`.
- **One Intern per session.** A second launch hands its documents to the
  running one over the D-Bus session bus and exits.
- **Instead of Send to.** The menu entry accepts files, so a file manager's
  **Open With** offers Intern and documents can be dropped on its launcher; from
  a terminal, `intern ~/Downloads/*.pdf` queues them. Intern registers for no
  document types, so it never becomes the application that opens them.
- **llama-server and the worker never outlive Intern** - a crash, a `SIGKILL`,
  or signing out included - and they run ten steps of niceness below it, the
  counterpart of Windows' below-normal priority. See
  [architecture](architecture.md).
- **The hosted model's key** lives in the Secret Service. Where none is running
  or it is locked - a minimal session, a server, an agent's VM - Settings says
  so (`SECRET_SERVICE_UNAVAILABLE`) and nothing else is affected; the local
  model needs no key. In a headless session `gnome-keyring-daemon --unlock`
  inside `dbus-run-session` provides one.
- **Updates.** Linux builds do not check for or install updates: no Linux
  build is in the signed release manifest, so the updater is not even
  registered, and Settings says to install the newer package instead. The
  only requests a Linux install makes are the model download and anything you
  turn on yourself.

## Known limitations

- **No Microsoft sign-in, OneDrive client or SharePoint deployment.** There is
  no OneDrive sync client for Linux, so the verified team Inbox, Microsoft
  upload verification, and the provisioned SharePoint setup are Windows-only.
  Folder setup lists no synced locations unless a `~/OneDrive…` folder exists
  (as a third-party client such as `onedrive` makes); a watched folder is
  otherwise an ordinary local folder.
- **Only the `.deb` is built and tested.** The app finds its runtime inside an
  AppImage too (`$APPDIR/usr/lib/Intern/runtime`), but no AppImage is built: it
  would carry its own copy of WebKitGTK, which is large and breaks on systems
  whose graphics stack differs, and its tooling is downloaded unpinned at
  bundle time.
- **x86-64 only**, and glibc 2.39 or newer for a package built on Ubuntu 24.04.
- **Tesseract is the distribution's**, so OCR fallback output can differ by a
  character from Windows; see above.
