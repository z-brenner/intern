//! Auditable laboratory runner. Local immutable inputs; no script interpreter or network bridge.
use intern_bench::qualification::{Asset, file_hash, relative, verified_assets};
use intern_engine::benchmark_transport::BenchmarkTransport;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    profile: String,
    #[serde(default)]
    body: Option<Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema_version: u32,
    run_id: String,
    files: Vec<Asset>,
    server: String,
    bench: String,
    worker: String,
    corpus: String,
    gold: String,
    manifest: String,
    model_sha256: String,
    cases: Vec<Case>,
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}
fn save(path: &Path, data: &[u8]) -> Result<(), String> {
    let mut f = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    f.write_all(data)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())
}
fn value_file(path: &Path, v: &Value) -> Result<(), String> {
    save(
        path,
        &serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?,
    )
}
fn read_plan(path: &Path, expected: &str) -> Result<Plan, String> {
    if file_hash(path)? != expected {
        return Err("plan SHA-256 mismatch".into());
    }
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut b = Vec::new();
    f.take(2_000_001)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    if b.len() > 2_000_000 {
        return Err("plan too large".into());
    }
    let p: Plan = serde_json::from_slice(&b).map_err(|e| e.to_string())?;
    let mut ids = std::collections::BTreeSet::new();
    if p.schema_version != 1 || !identifier(&p.run_id) || p.cases.is_empty() || p.cases.len() > 18 {
        return Err("invalid plan bounds".into());
    }
    for c in &p.cases {
        if !identifier(&c.id) || !ids.insert(&c.id) {
            return Err("invalid/duplicate case id".into());
        }
    }
    Ok(p)
}
#[cfg(windows)]
#[allow(unsafe_code)]
mod owned {
    use super::*;
    use std::os::windows::{io::AsRawHandle, process::CommandExt};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First,
                Thread32Next,
            },
            JobObjects::*,
            ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
            Threading::{CREATE_SUSPENDED, OpenThread, ResumeThread, THREAD_SUSPEND_RESUME},
        },
    };
    pub struct Process {
        pub child: Child,
        job: HANDLE,
    }
    impl Process {
        pub fn spawn(cmd: &mut Command) -> Result<Self, String> {
            // No inherited job handle: closing this owner on any exit reaps the exact child tree.
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if job.is_null() {
                    return Err("cannot create owned job".into());
                }
                let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
                limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                if SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    std::ptr::from_ref(&limits).cast(),
                    size_of_val(&limits) as u32,
                ) == 0
                {
                    CloseHandle(job);
                    return Err("cannot configure owned job".into());
                }
                let mut child = match cmd.creation_flags(CREATE_SUSPENDED).spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        CloseHandle(job);
                        return Err(e.to_string());
                    }
                };
                if AssignProcessToJobObject(job, child.as_raw_handle() as HANDLE) == 0 {
                    let _ = child.kill();
                    let _ = child.wait();
                    CloseHandle(job);
                    return Err("cannot bind child to owned job".into());
                }
                let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
                let mut resumed = false;
                if snapshot != INVALID_HANDLE_VALUE {
                    let mut entry = THREADENTRY32::default();
                    entry.dwSize = size_of_val(&entry) as u32;
                    let mut found = Thread32First(snapshot, &mut entry);
                    while found != 0 {
                        if entry.th32OwnerProcessID == child.id() {
                            let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                            if !thread.is_null() {
                                resumed = ResumeThread(thread) == 1;
                                CloseHandle(thread);
                            }
                            break;
                        }
                        found = Thread32Next(snapshot, &mut entry);
                    }
                    CloseHandle(snapshot);
                }
                if !resumed {
                    let _ = child.kill();
                    let _ = child.wait();
                    CloseHandle(job);
                    return Err("cannot resume job-bound child".into());
                }
                Ok(Self { child, job })
            }
        }
        pub fn peak(&self) -> Option<usize> {
            unsafe {
                let mut p = PROCESS_MEMORY_COUNTERS::default();
                p.cb = size_of_val(&p) as u32;
                (GetProcessMemoryInfo(
                    self.child.as_raw_handle() as HANDLE,
                    &mut p,
                    size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                ) != 0)
                    .then_some(p.PeakWorkingSetSize)
            }
        }
    }
    impl Drop for Process {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.job);
            }
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
#[cfg(not(windows))]
mod owned {
    use super::*;
    pub struct Process {
        pub child: Child,
    }
    impl Process {
        pub fn spawn(_: &mut Command) -> Result<Self, String> {
            Err("qualification execution requires Windows".into())
        }
        pub fn peak(&self) -> Option<usize> {
            None
        }
    }
}
fn builtin_output(cmd: &mut Command) -> Result<std::process::Output, String> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let started = Instant::now();
    loop {
        if child.try_wait().map_err(|e| e.to_string())?.is_some() {
            return child.wait_with_output().map_err(|e| e.to_string());
        }
        if started.elapsed() > Duration::from_secs(2) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Windows status command exceeded two-second deadline".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}
fn defender_event() -> Result<(u64, Vec<u8>), String> {
    if !cfg!(windows) {
        return Err("Defender verification requires Windows".into());
    }
    let o = builtin_output(Command::new(r"C:\Windows\System32\wevtutil.exe").args([
        "qe",
        "Microsoft-Windows-Windows Defender/Operational",
        "/q:*[System[(EventID=1116)]]",
        "/c:1",
        "/rd:true",
        "/f:xml",
    ]))?;
    if !o.status.success() {
        return Err("cannot read Defender operational log".into());
    }
    let text = if o.stdout.starts_with(&[255, 254]) {
        String::from_utf16_lossy(
            &o.stdout[2..]
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect::<Vec<_>>(),
        )
    } else {
        String::from_utf8_lossy(&o.stdout).into_owned()
    };
    let id = if text.trim().is_empty() {
        0
    } else {
        text.split("<EventRecordID>")
            .nth(1)
            .and_then(|x| x.split('<').next())
            .and_then(|x| x.parse().ok())
            .ok_or("unreadable Defender event record")?
    };
    Ok((id, o.stdout))
}
fn monitor(out: &Path) -> Result<(Arc<AtomicBool>, std::thread::JoinHandle<()>, u64), String> {
    let (baseline, xml) = defender_event()?;
    save(&out.join("defender-start.xml"), &xml)?;
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let out = out.to_owned();
    let handle = std::thread::spawn(move || {
        while !flag.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_secs(2));
            if flag.load(Ordering::SeqCst) {
                break;
            }
            match defender_event() {
                Ok((id, xml)) if id == baseline => {
                    let _ = xml;
                }
                result => {
                    let reason = match result {
                        Ok((_, xml)) => {
                            let _ = save(&out.join("defender-new-alert.xml"), &xml);
                            "new Defender detection".to_owned()
                        }
                        Err(e) => e,
                    };
                    let _ = value_file(&out.join("security-stop.json"), &json!({"reason":reason}));
                    eprintln!("SECURITY STOP: {reason}");
                    std::process::exit(86);
                }
            }
        }
    });
    Ok((stop, handle, baseline))
}
fn verify_listener(expected: u32) -> Result<bool, String> {
    let o = builtin_output(
        Command::new(r"C:\Windows\System32\netstat.exe").args(["-ano", "-p", "tcp"]),
    )?;
    if !o.status.success() {
        return Err("cannot verify model listener owner".into());
    }
    for line in String::from_utf8_lossy(&o.stdout).lines() {
        let p = line.split_whitespace().collect::<Vec<_>>();
        if p.len() >= 5 && p[1] == "127.0.0.1:48761" && p[3] == "LISTENING" {
            return if p[4].parse::<u32>().ok() == Some(expected) {
                Ok(true)
            } else {
                Err("model port belongs to an unexpected process".into())
            };
        }
    }
    Ok(false)
}
fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(150))
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .tls_built_in_native_certs(false)
        .build()
        .map_err(|e| e.to_string())
}
fn output_file(path: &Path) -> Result<std::fs::File, String> {
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())
}
fn run() -> Result<(), String> {
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    if a.len() != 3 {
        return Err("usage: intern-qualify PLAN.json PLAN_SHA256 MODEL.gguf".into());
    }
    let plan_path = PathBuf::from(&a[0]);
    let p = read_plan(&plan_path, &a[1])?;
    let root = plan_path
        .parent()
        .ok_or("plan parent missing")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    verified_assets(&root, &p.files)?;
    let allowed = |name: &str| -> Result<PathBuf, String> {
        if !p.files.iter().any(|f| f.path == name) {
            return Err(format!("unlisted executable/input: {name}"));
        }
        relative(&root, name)
    };
    let server = allowed(&p.server)?;
    let bench = allowed(&p.bench)?;
    let worker = allowed(&p.worker)?;
    let gold = allowed(&p.gold)?;
    let manifest = allowed(&p.manifest)?;
    for c in &p.cases {
        let _ = allowed(&c.profile)?;
    }
    let corpus = relative(&root, &p.corpus)?;
    let model = PathBuf::from(&a[2])
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if file_hash(&model)? != p.model_sha256 {
        return Err("model SHA-256 mismatch".into());
    }
    let out = std::env::temp_dir().join(format!("InternNative-{}", p.run_id));
    std::fs::create_dir(&out).map_err(|e| format!("output must be new: {e}"))?;
    value_file(
        &out.join("verified.json"),
        &json!({"plan_sha256":a[1],"model_sha256":p.model_sha256,"runner_pid":std::process::id(),"network":"Windows loopback only; guest NIC must be disabled by host","files":p.files}),
    )?;
    println!("Verified inputs. Output: {}", out.display());
    let (stop, monitor_thread, defender_baseline) = monitor(&out)?;
    // Session-only key: no file, console output, manifest or capture contains it.
    let key = rand::random::<[u8; 32]>()
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect::<String>();
    let mut cmd = Command::new(server);
    cmd.args([
        "--host",
        "127.0.0.1",
        "--port",
        "48761",
        "--api-key",
        &key,
        "--model",
    ])
    .arg(&model)
    .args([
        "--alias",
        "intern-local",
        "--parallel",
        "1",
        "--ctx-size",
        "8192",
        "--n-gpu-layers",
        "0",
        "--threads",
        "2",
        "--threads-batch",
        "2",
        "--cache-ram",
        "64",
        "--jinja",
        "--no-webui",
        "--no-mmproj",
    ]);
    cmd.stdout(Stdio::from(output_file(&out.join("server.log"))?))
        .stderr(Stdio::from(output_file(&out.join("server.err"))?));
    let mut model_process = owned::Process::spawn(&mut cmd)?;
    let http = client()?;
    let startup = Instant::now();
    loop {
        if model_process
            .child
            .try_wait()
            .map_err(|e| e.to_string())?
            .is_some()
        {
            return Err("owned model server exited during startup".into());
        }
        if verify_listener(model_process.child.id())?
            && http
                .get("http://127.0.0.1:48761/health")
                .bearer_auth(&key)
                .timeout(Duration::from_secs(2))
                .send()
                .is_ok_and(|r| r.status().is_success())
        {
            break;
        }
        if startup.elapsed() > Duration::from_secs(150) {
            return Err("model startup deadline".into());
        }
        std::thread::sleep(Duration::from_millis(400));
    }
    value_file(
        &out.join("server-ready.json"),
        &json!({"pid":model_process.child.id(),"startup_seconds":startup.elapsed().as_secs_f64()}),
    )?;
    let whole = Instant::now();
    let mut attempted = 0;
    for (i, c) in p.cases.iter().enumerate() {
        if whole.elapsed() > Duration::from_secs(360) {
            break;
        }
        let case = out.join(&c.id);
        std::fs::create_dir(&case).map_err(|e| e.to_string())?;
        let start = Instant::now();
        let profile = allowed(&c.profile)?;
        if !verify_listener(model_process.child.id())? {
            return Err("owned model listener missing".into());
        }
        println!("CASE {}", c.id);
        let outcome: Result<Value, String> = (|| {
            if let Some(body) = &c.body {
                let transport = BenchmarkTransport::load(&profile, &case.join("wire"))?;
                let (wire, id) = transport.prepare(body, "raw-calibration")?;
                let t = Instant::now();
                let response = http
                    .post("http://127.0.0.1:48761/v1/chat/completions")
                    .bearer_auth(&key)
                    .header("Content-Type", "application/json")
                    .body(wire)
                    .send()
                    .map_err(|e| e.to_string())?;
                let status = response.status();
                let mut bytes = Vec::new();
                response
                    .take(2_097_153)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                if bytes.len() > 2_097_152 {
                    return Err("oversized response".into());
                }
                transport.response(id, status.as_u16(), &bytes, t.elapsed().as_secs_f64())?;
                return Ok(json!({"http_status":status.as_u16(),"ok":status.is_success()}));
            }
            let mut cmd = Command::new(&bench);
            cmd.arg("run")
                .arg("--corpus")
                .arg(&corpus)
                .arg("--gold")
                .arg(&gold)
                .arg("--manifest")
                .arg(&manifest)
                .arg("--worker")
                .arg(&worker)
                .args([
                    "--endpoint",
                    "http://127.0.0.1:48761/v1/chat/completions",
                    "--api-key",
                    &key,
                    "--model-id",
                    "intern-local",
                    "--model-path",
                ])
                .arg(&model)
                .args([
                    "--only",
                    &c.id,
                    "--pipeline",
                    "evidence",
                    "--id-style",
                    "stable",
                    "--context-tokens",
                    "8192",
                    "--server-pid",
                ])
                .arg(model_process.child.id().to_string());
            cmd.arg("--bench-profile")
                .arg(&profile)
                .arg("--wire-output")
                .arg(case.join("wire"))
                .arg("--record")
                .arg(case.join("recording.json"))
                .arg("--output")
                .arg(case.join("report.json"))
                .arg("--markdown")
                .arg(case.join("report.md"));
            if i > 0 {
                cmd.arg("--no-warmup");
            }
            cmd.env(
                "INTERN_RUNTIME_DIR",
                worker.parent().ok_or("worker parent missing")?,
            )
            .stdout(Stdio::from(output_file(&case.join("bench.log"))?))
            .stderr(Stdio::from(output_file(&case.join("bench.err"))?));
            let mut child = owned::Process::spawn(&mut cmd)?;
            loop {
                if let Some(status) = child.child.try_wait().map_err(|e| e.to_string())? {
                    let report_path = case.join("report.json");
                    let mut report_bytes = Vec::new();
                    std::fs::File::open(&report_path)
                        .map_err(|e| e.to_string())?
                        .take(20_000_001)
                        .read_to_end(&mut report_bytes)
                        .map_err(|e| e.to_string())?;
                    if report_bytes.len() > 20_000_000 {
                        return Err("oversized native report".into());
                    }
                    let report: Value =
                        serde_json::from_slice(&report_bytes).map_err(|e| e.to_string())?;
                    return Ok(
                        json!({"exit_code":status.code(),"ok":status.success() && intern_bench::qualification::report_completed(&report,&c.id),"report_sha256":file_hash(&report_path)?,"bench_peak_working_set_bytes":child.peak()}),
                    );
                }
                if model_process
                    .child
                    .try_wait()
                    .map_err(|e| e.to_string())?
                    .is_some()
                {
                    return Err("owned model server exited".into());
                }
                if start.elapsed() > Duration::from_secs(240) {
                    return Err("case deadline exceeded".into());
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        })();
        let success = outcome.as_ref().is_ok_and(|v| v["ok"] == true);
        value_file(
            &case.join("terminal.json"),
            &json!({"id":c.id,"elapsed_seconds":start.elapsed().as_secs_f64(),"outcome":outcome.as_ref().ok(),"error":outcome.as_ref().err(),"server_peak_working_set_bytes":model_process.peak()}),
        )?;
        attempted += 1;
        if !success {
            return Err(format!(
                "case {} stopped; preserve all partial evidence",
                c.id
            ));
        }
    }
    let peak = model_process.peak();
    drop(model_process);
    stop.store(true, Ordering::SeqCst);
    monitor_thread
        .join()
        .map_err(|_| "Defender monitor panicked".to_owned())?;
    intern_bench::qualification::final_security_check(defender_baseline, defender_event(), &out)?;
    value_file(
        &out.join("terminal.json"),
        &json!({"run_id":p.run_id,"attempted":attempted,"planned":p.cases.len(),"elapsed_seconds":whole.elapsed().as_secs_f64(),"server_peak_working_set_bytes":peak,"complete":attempted==p.cases.len()}),
    )?;
    println!(
        "BATCH STOPPED; {} cases. Collect only after guest shutdown.",
        attempted
    );
    intern_bench::qualification::require_complete(attempted, p.cases.len())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("STOP: {e}");
        std::process::exit(1)
    }
}
