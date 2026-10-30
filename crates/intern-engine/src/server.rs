//! Supervision of the local llama.cpp server process.
//!
//! The server binds a random loopback port with a random bearer key and is
//! kept warm between documents. Stopping it is not left to `Drop`: on Windows
//! the process joins the job object in this crate's `process` module, so it
//! dies with Intern however Intern ends, including the `std::process::exit`
//! the window close and the updater both leave through.

use std::{
    fmt,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use rand::{RngCore, rngs::OsRng};
use reqwest::blocking::Client;

use crate::error::{EngineError, EngineErrorCode, EngineResult};

const HEALTH_POLL_INTERVAL: Duration = Duration::from_millis(100);
const HEALTH_CONFIRMATION_DELAY: Duration = Duration::from_millis(500);
const MAX_SPAWN_ATTEMPTS: usize = 3;
/// The server's standard error, under the log directory when one is set.
const SERVER_LOG: &str = "llama-server.log";

/// Flags every local model is launched with, each measured end to end on
/// Intern's own requests in the 77-document evaluation
/// (`bench/reports/bonsai-linux-cpu-2026-10-27`, Prism `b10826`, which has
/// every one of them):
///
/// * `--cache-ram 0`: llama-server otherwise keeps every finished prompt in
///   an 8 GiB host-memory cache. Intern's prompts never repeat - only the
///   shared instruction prefix does, and the slot reuses that without the
///   cache - so the server grew by 80-280 MB a document, toward 10 GB over a
///   long background session. Off, the replies were identical (12 of 12 for
///   each model), the prefix was still reused, and the time rose 1-2%.
/// * `-ub 256`: on the AVX2-only CPU code path most laptops load, documents
///   took 13-14% less time than with the default 512, with the same replies;
///   on AVX-512 it made no difference. On upstream `b11946`, which the
///   accurate tier runs on, 128, 256 and 512 all fell within run-to-run noise
///   on the AVX2 and AVX-VNNI paths, and 128 or 512 changed most replies, so
///   256 stays (`bench/reports/speed-linux-cpu-2026-10-29`).
/// * `--prio -1 --poll 0`: low thread priority, and threads that sleep
///   rather than spin while waiting for work. Intern reads in the background,
///   and together these cost nothing measurable (-2% to +3% against the
///   default flags, within the run-to-run noise) with identical replies. On
///   upstream, spinning (`--poll 50`) was no faster either.
pub const MEASURED_FLAGS: &[&str] = &[
    "--cache-ram",
    "0",
    "-ub",
    "256",
    "--prio",
    "-1",
    "--poll",
    "0",
];

/// Context window. Large enough for the distillation budget plus the prompt,
/// small enough that the KV cache stays a few hundred megabytes on CPU.
pub const CONTEXT_TOKENS: u32 = 8_192;

/// Threads to give the model.
///
/// On a hyper-threaded laptop, llama.cpp scales with physical cores, not
/// logical ones, and taking every core makes the rest of Windows stutter.
/// Half the logical count is the physical count on the machines Intern targets,
/// and it leaves the other half for whatever else the user is doing.
pub fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map(|value| value.get() / 2)
        .unwrap_or(4)
        .clamp(2, 12)
}

/// How the model process should be launched.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerOptions {
    pub threads: usize,
    pub context_tokens: u32,
    pub startup_timeout: Duration,
    /// Flags for one model in particular, after everything above. Chosen per
    /// local model tier in [`crate::tiers::server_options`], which is the
    /// one place they are set.
    pub extra_arguments: Vec<String>,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            threads: default_threads(),
            context_tokens: CONTEXT_TOKENS,
            startup_timeout: Duration::from_secs(180),
            extra_arguments: Vec::new(),
        }
    }
}

pub trait ProcessControl: Send {
    fn has_exited(&mut self) -> EngineResult<bool>;
    fn terminate_and_wait(&mut self) -> EngineResult<()>;
}

pub trait ProcessLauncher: Send + Sync {
    fn launch(
        &self,
        executable: &Path,
        arguments: &[String],
    ) -> EngineResult<Box<dyn ProcessControl>>;
}

pub trait PortAllocator: Send + Sync {
    fn next_port(&self) -> EngineResult<u16>;
}

pub trait HealthProbe: Send + Sync {
    fn is_healthy(&self, endpoint: &str, api_key: &str) -> bool;
}

pub struct LlamaServer {
    process: Mutex<Box<dyn ProcessControl>>,
    endpoint: String,
    api_key: String,
    health_probe: Arc<dyn HealthProbe>,
}

impl fmt::Debug for LlamaServer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LlamaServer")
            .field("endpoint", &self.endpoint)
            .field("api_key", &"[redacted]")
            .finish_non_exhaustive()
    }
}

impl LlamaServer {
    pub fn start(
        executable: &Path,
        model: &Path,
        projector: Option<&Path>,
        options: &ServerOptions,
    ) -> EngineResult<Self> {
        Self::start_with(
            executable,
            model,
            projector,
            options,
            HEALTH_CONFIRMATION_DELAY,
            &StdProcessLauncher::new(),
            &EphemeralPortAllocator,
            Arc::new(ReqwestHealthProbe::new()?),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start_with(
        executable: &Path,
        model: &Path,
        projector: Option<&Path>,
        options: &ServerOptions,
        health_confirmation_delay: Duration,
        launcher: &dyn ProcessLauncher,
        ports: &dyn PortAllocator,
        health_probe: Arc<dyn HealthProbe>,
    ) -> EngineResult<Self> {
        require_file(executable)?;
        require_file(model)?;
        if let Some(projector) = projector {
            require_file(projector)?;
        }
        if options.startup_timeout.is_zero() {
            return Err(start_failed());
        }

        let deadline = Instant::now() + options.startup_timeout;
        for attempt in 0..MAX_SPAWN_ATTEMPTS {
            if Instant::now() >= deadline {
                return Err(unhealthy());
            }
            let port = ports.next_port()?;
            let api_key = random_api_key();
            let arguments = Self::arguments_for(model, projector, port, &api_key, options);
            let mut process = match launcher.launch(executable, &arguments) {
                Ok(process) => process,
                Err(_) if attempt + 1 < MAX_SPAWN_ATTEMPTS => continue,
                Err(_) => return Err(start_failed()),
            };
            let endpoint = format!("http://127.0.0.1:{port}");

            let startup_state = match confirm_startup(
                process.as_mut(),
                health_probe.as_ref(),
                &endpoint,
                &api_key,
                deadline,
                health_confirmation_delay,
            ) {
                Ok(state) => state,
                Err(error) => {
                    process.terminate_and_wait()?;
                    return Err(error);
                }
            };

            match startup_state {
                StartupState::Ready => {
                    return Ok(Self {
                        process: Mutex::new(process),
                        endpoint,
                        api_key,
                        health_probe,
                    });
                }
                StartupState::Exited => {
                    process.terminate_and_wait()?;
                    if attempt + 1 == MAX_SPAWN_ATTEMPTS {
                        return Err(start_failed());
                    }
                }
                StartupState::TimedOut => {
                    process.terminate_and_wait()?;
                    return Err(unhealthy());
                }
            }
        }
        Err(start_failed())
    }

    pub fn arguments_for(
        model: &Path,
        projector: Option<&Path>,
        port: u16,
        api_key: &str,
        options: &ServerOptions,
    ) -> Vec<String> {
        let mut arguments = vec![
            "--host".into(),
            "127.0.0.1".into(),
            "--port".into(),
            port.to_string(),
            "--api-key".into(),
            api_key.into(),
            "--model".into(),
            model.to_string_lossy().into_owned(),
            "--parallel".into(),
            "1".into(),
            "--ctx-size".into(),
            options.context_tokens.to_string(),
            "--n-gpu-layers".into(),
            "0".into(),
            "--threads".into(),
            options.threads.to_string(),
            "--threads-batch".into(),
            options.threads.to_string(),
            // The model's own chat template is required for the
            // enable_thinking switch to reach hybrid-reasoning models.
            "--jinja".into(),
            "--no-webui".into(),
        ];
        arguments.extend(MEASURED_FLAGS.iter().map(|flag| (*flag).to_owned()));
        match projector {
            Some(projector) => {
                arguments.push("--mmproj".into());
                arguments.push(projector.to_string_lossy().into_owned());
            }
            // Loading a vision projector Intern will not use costs hundreds of
            // megabytes of resident memory on every document.
            None => arguments.push("--no-mmproj".into()),
        }
        arguments.extend(options.extra_arguments.iter().cloned());
        arguments
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn completion_endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.endpoint)
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn health(&self) -> bool {
        let Ok(mut process) = self.process.lock() else {
            return false;
        };
        if process.has_exited().unwrap_or(true) {
            return false;
        }
        let healthy = self.health_probe.is_healthy(&self.endpoint, &self.api_key);
        healthy && !process.has_exited().unwrap_or(true)
    }

    pub fn stop(&self) -> EngineResult<()> {
        self.process
            .lock()
            .map_err(|_| start_failed())?
            .terminate_and_wait()
    }
}

impl Drop for LlamaServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

enum StartupState {
    Ready,
    Exited,
    TimedOut,
}

fn confirm_startup(
    process: &mut dyn ProcessControl,
    health: &dyn HealthProbe,
    endpoint: &str,
    api_key: &str,
    deadline: Instant,
    confirmation_delay: Duration,
) -> EngineResult<StartupState> {
    loop {
        if process.has_exited()? {
            return Ok(StartupState::Exited);
        }
        let healthy = health.is_healthy(endpoint, api_key);
        if process.has_exited()? {
            return Ok(StartupState::Exited);
        }
        if healthy {
            if !confirmation_delay.is_zero() {
                thread::sleep(
                    confirmation_delay.min(deadline.saturating_duration_since(Instant::now())),
                );
            }
            if process.has_exited()? {
                return Ok(StartupState::Exited);
            }
            let confirmed = health.is_healthy(endpoint, api_key);
            if process.has_exited()? {
                return Ok(StartupState::Exited);
            }
            if confirmed {
                return Ok(StartupState::Ready);
            }
        }
        if Instant::now() >= deadline {
            return Ok(StartupState::TimedOut);
        }
        thread::sleep(HEALTH_POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now())));
    }
}

struct StdChildProcess(Child);

impl ProcessControl for StdChildProcess {
    fn has_exited(&mut self) -> EngineResult<bool> {
        self.0
            .try_wait()
            .map(|status| status.is_some())
            .map_err(|_| start_failed())
    }

    fn terminate_and_wait(&mut self) -> EngineResult<()> {
        if self.0.try_wait().map_err(|_| start_failed())?.is_some() {
            return Ok(());
        }
        let kill_result = self.0.kill();
        let wait_result = self.0.wait();
        if wait_result.is_ok() {
            return Ok(());
        }
        kill_result.map_err(|_| start_failed())?;
        Err(start_failed())
    }
}

struct StdProcessLauncher {
    /// Where the server's standard error is kept; nowhere when `None`.
    log_directory: Option<PathBuf>,
}

impl StdProcessLauncher {
    fn new() -> Self {
        Self {
            log_directory: crate::logs::log_directory().map(Path::to_path_buf),
        }
    }
}

impl ProcessLauncher for StdProcessLauncher {
    fn launch(
        &self,
        executable: &Path,
        arguments: &[String],
    ) -> EngineResult<Box<dyn ProcessControl>> {
        let mut command = command_in_own_folder(executable)?;
        // Standard error is where llama-server says why it could not start:
        // missing CPU features, a quarantined runtime library, a model that
        // will not load. At its default verbosity it logs no prompt text and
        // not its `--api-key`; it must never be started with `-v`.
        command
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(crate::logs::stderr_for(
                self.log_directory.as_deref(),
                SERVER_LOG,
            ));
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(crate::process::sidecar_creation_flags());
        }
        let child = crate::process::spawn_tied(command).map_err(|_| start_failed())?;
        Ok(Box::new(StdChildProcess(child)))
    }
}

/// A command for `executable` that runs in the executable's own folder.
///
/// ggml loads its `ggml-cpu-*` backends from the executable's folder and
/// from the current directory. On Windows Intern's current directory is often
/// the install root, where Prism's CPU backends sit under the same names as
/// upstream's; in its own folder, each llama-server finds its own.
///
/// A path with a folder is made absolute first, since a relative program is
/// found from the child's directory on some platforms and the parent's on
/// others. A bare name is left to the search path, from where Intern is.
fn command_in_own_folder(executable: &Path) -> EngineResult<Command> {
    let has_folder = executable
        .parent()
        .is_some_and(|folder| !folder.as_os_str().is_empty());
    if !has_folder {
        return Ok(Command::new(executable));
    }
    let executable = std::path::absolute(executable).map_err(|_| start_failed())?;
    let mut command = Command::new(&executable);
    if let Some(folder) = executable.parent() {
        command.current_dir(folder);
    }
    Ok(command)
}

struct EphemeralPortAllocator;

impl PortAllocator for EphemeralPortAllocator {
    fn next_port(&self) -> EngineResult<u16> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|_| start_failed())?;
        listener
            .local_addr()
            .map(|address| address.port())
            .map_err(|_| start_failed())
    }
}

struct ReqwestHealthProbe {
    client: Client,
}

impl ReqwestHealthProbe {
    fn new() -> EngineResult<Self> {
        // Loopback HTTP only; see `ModelClient::new` for why the operating
        // system's certificate store is left unread.
        let client = Client::builder()
            .connect_timeout(Duration::from_millis(500))
            .timeout(Duration::from_secs(1))
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .tls_built_in_native_certs(false)
            .build()
            .map_err(|_| start_failed())?;
        Ok(Self { client })
    }
}

impl HealthProbe for ReqwestHealthProbe {
    fn is_healthy(&self, endpoint: &str, api_key: &str) -> bool {
        self.client
            .get(format!("{endpoint}/health"))
            .bearer_auth(api_key)
            .send()
            .is_ok_and(|response| response.status().is_success())
    }
}

fn random_api_key() -> String {
    use std::fmt::Write as _;
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let mut key = String::with_capacity(64);
    for byte in bytes {
        let _ = write!(key, "{byte:02x}");
    }
    key
}

fn require_file(path: &Path) -> EngineResult<()> {
    path.is_file().then_some(()).ok_or_else(start_failed)
}

const fn start_failed() -> EngineError {
    EngineError::new(
        EngineErrorCode::ModelServerStartFailed,
        "local model server could not be started",
    )
}

const fn unhealthy() -> EngineError {
    EngineError::new(
        EngineErrorCode::ModelServerUnhealthy,
        "local model server did not become healthy",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> ServerOptions {
        ServerOptions {
            threads: 6,
            context_tokens: 8_192,
            startup_timeout: Duration::from_secs(30),
            extra_arguments: Vec::new(),
        }
    }

    /// A model's own flags come last, after everything every launch sets.
    #[test]
    fn extra_arguments_follow_the_common_ones() {
        let options = ServerOptions {
            extra_arguments: vec!["--flash-attn".into(), "on".into()],
            ..options()
        };
        let arguments =
            LlamaServer::arguments_for(Path::new("m.gguf"), None, 51_000, "key", &options);
        assert_eq!(&arguments[arguments.len() - 2..], ["--flash-attn", "on"]);
        assert!(arguments.contains(&"--no-mmproj".to_owned()));
    }

    /// The flags the evaluation measured are on every launch, and the batch
    /// threads always match the generation threads.
    #[test]
    fn every_launch_carries_the_measured_flags() {
        let arguments =
            LlamaServer::arguments_for(Path::new("m.gguf"), None, 51_000, "key", &options());
        let value = |flag: &str| {
            arguments
                .iter()
                .position(|argument| argument == flag)
                .map(|at| arguments[at + 1].as_str())
        };
        assert_eq!(value("--cache-ram"), Some("0"));
        assert_eq!(value("-ub"), Some("256"));
        assert_eq!(value("--prio"), Some("-1"));
        assert_eq!(value("--poll"), Some("0"));
        assert_eq!(value("--threads"), Some("6"));
        assert_eq!(value("--threads-batch"), Some("6"));
        assert_eq!(value("--parallel"), Some("1"));
        assert_eq!(value("--ctx-size"), Some("8192"));
        assert_eq!(value("--n-gpu-layers"), Some("0"));
        // A cold disk took up to 117 seconds to start a model in the
        // evaluation; the health wait allows more than that.
        assert!(ServerOptions::default().startup_timeout >= Duration::from_secs(180));
    }

    #[test]
    fn a_text_only_launch_refuses_to_load_a_projector() {
        let arguments =
            LlamaServer::arguments_for(Path::new("m.gguf"), None, 51_000, "key", &options());
        assert!(arguments.contains(&"--no-mmproj".to_owned()));
        assert!(!arguments.contains(&"--mmproj".to_owned()));
        assert!(arguments.contains(&"--threads".to_owned()));
        assert!(arguments.contains(&"6".to_owned()));
        assert!(arguments.contains(&"127.0.0.1".to_owned()));
    }

    #[test]
    fn a_vision_launch_loads_the_projector() {
        let arguments = LlamaServer::arguments_for(
            Path::new("m.gguf"),
            Some(Path::new("p.gguf")),
            51_000,
            "key",
            &options(),
        );
        assert!(arguments.contains(&"--mmproj".to_owned()));
        assert!(!arguments.contains(&"--no-mmproj".to_owned()));
    }

    /// The server's standard error is the only account of why it would not
    /// start, so with a log directory it is kept - in a file that cannot grow
    /// without bound - and without one it goes nowhere, as before.
    #[cfg(unix)]
    #[test]
    fn stderr_goes_to_capped_log_when_directory_set() {
        let directory = tempfile::tempdir().unwrap();
        let logs = directory.path().join("logs");
        std::fs::create_dir_all(&logs).unwrap();
        std::fs::write(logs.join(SERVER_LOG), vec![b'x'; 300 * 1024]).unwrap();

        let launcher = StdProcessLauncher {
            log_directory: Some(logs.clone()),
        };
        let mut process = launcher
            .launch(
                Path::new("/bin/sh"),
                &[
                    "-c".to_owned(),
                    "echo 'error: unknown model architecture' >&2".to_owned(),
                ],
            )
            .expect("a shell");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !process.has_exited().unwrap() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            std::fs::read_to_string(logs.join(SERVER_LOG)).unwrap(),
            "error: unknown model architecture\n",
            "the oversized log was emptied before this start wrote to it"
        );

        // No directory: the launch is unchanged, and nothing is written.
        let launcher = StdProcessLauncher {
            log_directory: None,
        };
        let mut process = launcher
            .launch(
                Path::new("/bin/sh"),
                &["-c".to_owned(), "echo lost >&2".to_owned()],
            )
            .expect("a shell");
        while !process.has_exited().unwrap() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(std::fs::read_dir(&logs).unwrap().count(), 1);
    }

    /// A server runs in its own folder, where ggml finds its own CPU backends
    /// rather than another runtime's in Intern's current directory - also
    /// when it is named by a relative path. A program named without a folder
    /// is still found on the search path, and runs where Intern is.
    #[cfg(unix)]
    #[test]
    fn a_server_runs_in_its_own_folder() {
        let here = std::env::current_dir().unwrap();
        let directory = tempfile::tempdir().unwrap();
        // Deeper than the directory the tests run in, so that the relative
        // path below - up to the root from here, and down again - names
        // nothing from the server's own folder.
        let mut folder = directory.path().to_path_buf();
        while folder.components().count() <= here.components().count() {
            folder.push("deeper");
        }
        let folder = folder.join("llama-upstream");
        std::fs::create_dir_all(&folder).unwrap();
        // A shell under the server's name: nothing is written and then run.
        std::os::unix::fs::symlink("/bin/sh", folder.join("llama-server")).unwrap();
        let launcher = StdProcessLauncher {
            log_directory: None,
        };
        let record = directory.path().join("ran-in");
        let ran_in = |executable: &Path| {
            let _ = std::fs::remove_file(&record);
            let mut process = launcher
                .launch(
                    executable,
                    &["-c".to_owned(), format!("pwd -P > '{}'", record.display())],
                )
                .expect("a shell");
            let deadline = Instant::now() + Duration::from_secs(10);
            while !process.has_exited().unwrap() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            PathBuf::from(std::fs::read_to_string(&record).unwrap().trim_end())
        };
        let own = folder.canonicalize().unwrap();
        assert_eq!(ran_in(&folder.join("llama-server")), own);

        let relative = here
            .components()
            .skip(1)
            .map(|_| "..")
            .collect::<PathBuf>()
            .join(folder.strip_prefix("/").unwrap())
            .join("llama-server");
        assert!(relative.is_relative());
        assert_eq!(ran_in(&relative), own);

        assert_eq!(ran_in(Path::new("sh")), here.canonicalize().unwrap());
    }

    /// Windows sidecars open no console window and run below normal
    /// priority, so filing documents never makes the foreground stutter.
    #[test]
    fn sidecar_creation_flags_include_below_normal() {
        let flags = crate::process::sidecar_creation_flags();
        assert_eq!(
            flags & 0x0000_4000,
            0x0000_4000,
            "BELOW_NORMAL_PRIORITY_CLASS"
        );
        assert_eq!(flags & 0x0800_0000, 0x0800_0000, "CREATE_NO_WINDOW");
        assert_eq!(flags, 0x0800_4000, "nothing else");
    }

    #[test]
    fn the_thread_default_leaves_the_machine_usable() {
        let threads = default_threads();
        assert!((2..=12).contains(&threads));
    }

    /// The point of the job object: a child of a process that dies without
    /// running a single destructor must die too. The parent here leaves
    /// through `std::process::exit`, which is exactly how the window close and
    /// the updater's install step leave.
    #[cfg(windows)]
    #[test]
    fn a_launched_child_dies_with_its_parent() {
        let workspace = tempfile::tempdir().expect("a temporary directory");
        let announcement = workspace.path().join("child.pid");
        let parent = Command::new(std::env::current_exe().expect("the test binary"))
            .args([
                "server::tests::the_parent_that_leaves_without_running_destructors",
                "--exact",
                "--ignored",
            ])
            .env(CHILD_ANNOUNCEMENT, &announcement)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("the helper parent should run");
        assert_eq!(
            parent.code(),
            Some(ABRUPT_EXIT_CODE),
            "the helper parent should have left through process::exit"
        );

        let child = announced_pid(&announcement).expect("the child should have announced itself");
        let deadline = Instant::now() + Duration::from_secs(30);
        while is_running(child) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(100));
        }
        let survived = is_running(child);
        if survived {
            let _ = Command::new("taskkill.exe")
                .args(["/PID", &child.to_string(), "/F"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        assert!(
            !survived,
            "process {child} outlived the parent that launched it"
        );
    }

    /// The other half of the test above. Ignored so an ordinary run never
    /// executes it, and inert without the announcement path even then.
    #[cfg(windows)]
    #[test]
    #[ignore = "helper process for a_launched_child_dies_with_its_parent"]
    fn the_parent_that_leaves_without_running_destructors() {
        let Ok(announcement) = std::env::var(CHILD_ANNOUNCEMENT) else {
            return;
        };
        let script = format!(
            "Set-Content -LiteralPath '{announcement}' -Value $PID; Start-Sleep -Seconds 120"
        );
        let _child = StdProcessLauncher::new()
            .launch(
                Path::new("powershell.exe"),
                &[
                    "-NoProfile".to_owned(),
                    "-NonInteractive".to_owned(),
                    "-Command".to_owned(),
                    script,
                ],
            )
            .expect("the launcher should start a child");
        let deadline = Instant::now() + Duration::from_secs(60);
        while announced_pid(Path::new(&announcement)).is_none() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(50));
        }
        // No unwinding, no destructors, no stop() - the way Intern's own exit
        // paths leave.
        std::process::exit(ABRUPT_EXIT_CODE);
    }

    #[cfg(windows)]
    const CHILD_ANNOUNCEMENT: &str = "INTERN_TEST_CHILD_ANNOUNCEMENT";
    #[cfg(windows)]
    const ABRUPT_EXIT_CODE: i32 = 37;

    #[cfg(windows)]
    fn announced_pid(announcement: &Path) -> Option<u32> {
        std::fs::read_to_string(announcement)
            .ok()?
            .trim()
            .parse()
            .ok()
    }

    #[cfg(windows)]
    fn is_running(pid: u32) -> bool {
        let listing = Command::new("tasklist.exe")
            .args(["/NH", "/FI", &format!("PID eq {pid}")])
            .output()
            .expect("tasklist should run");
        String::from_utf8_lossy(&listing.stdout).contains(&pid.to_string())
    }
}
