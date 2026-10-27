//! Tying the processes Intern launches to the life of the process that
//! launched them.
//!
//! `Drop` is not a stop path anything can rely on. Closing the window with
//! background mode off, the tray's Quit item, and the updater's install step
//! all leave through `std::process::exit`, which runs no destructor at all,
//! and a panic or a hard crash leaves through even less. Each of those used to
//! leave llama-server holding well over a gigabyte until the machine was
//! rebooted, made the next launch start a second one, and left the updater
//! overwriting binaries that were still running.
//!
//! On Windows the kernel can do the reaping instead. A job object carrying
//! `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` terminates every process in it when
//! its last handle closes, and the last handle is this process's - closed by
//! the kernel however this process ends, deliberately or not. So every child
//! Intern spawns joins one process-wide job, and the ordinary stop paths
//! below become an optimisation rather than the only hope.
//!
//! On Linux the kernel does it by another route. A child that sets
//! `PR_SET_PDEATHSIG` before it runs anything is sent `SIGKILL` when its
//! parent ends - by `exit`, a panic, a crash, or a `SIGKILL` of its own. The
//! catch is that "parent" means the thread that started the child, not the
//! process: a llama-server started from one of the pipeline's short-lived
//! threads would be killed the moment that thread finished. So every child is
//! started from one thread kept for exactly that, which ends only with the
//! process. Each also starts [`SIDECAR_NICENESS`] steps below Intern's own
//! priority, the counterpart of the Windows priority class below.
//!
//! Everywhere else `Drop` remains the stop path it always was.

use std::{
    io,
    process::{Child, Command},
};

/// Windows `CREATE_NO_WINDOW`: a console sidecar opens no console window.
// Defined everywhere so the flags can be checked on any platform; only the
// Windows launchers pass them.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Windows `BELOW_NORMAL_PRIORITY_CLASS`. Inference and extraction are
/// background work: the person at the keyboard should never wait for a
/// document to be filed, and at normal priority a model holding half the
/// cores made typing in the next window stutter. They still get every cycle
/// nothing else wants.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;

/// The creation flags every sidecar - llama-server and `intern-worker` - is
/// launched with on Windows.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) const fn sidecar_creation_flags() -> u32 {
    CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS
}

/// How many steps of niceness below Intern every sidecar runs at on Linux:
/// the counterpart of [`BELOW_NORMAL_PRIORITY_CLASS`], for the same reason.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) const SIDECAR_NICENESS: i32 = 10;

/// Starts `command` as a child that dies with this process, whatever ends
/// this process.
///
/// Best effort on purpose. If the job cannot be created, the child cannot be
/// assigned to it, or there is no thread to start it from, the child still
/// stops the way it always did, and refusing to launch a model because of it
/// would help nobody.
#[cfg(windows)]
pub(crate) fn spawn_tied(mut command: Command) -> io::Result<Child> {
    let child = command.spawn()?;
    windows_job::assign(&child);
    Ok(child)
}

#[cfg(target_os = "linux")]
pub(crate) fn spawn_tied(command: Command) -> io::Result<Child> {
    linux_parent_death::spawn(command)
}

#[cfg(not(any(windows, target_os = "linux")))]
pub(crate) fn spawn_tied(mut command: Command) -> io::Result<Child> {
    command.spawn()
}

// Three system calls in the child between fork and exec, with no safe
// wrapper anywhere in the dependency tree. The rest of the crate keeps
// `#![deny(unsafe_code)]`.
#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
mod linux_parent_death {
    use std::{
        io,
        os::unix::process::CommandExt,
        process::{Child, Command},
        sync::{OnceLock, mpsc},
    };

    type Request = (Command, mpsc::SyncSender<io::Result<Child>>);

    /// The thread every child is started from, and so the parent whose end
    /// `PR_SET_PDEATHSIG` waits for. Nothing ever asks it to stop: its sender
    /// is kept here for as long as the process lives. `None` when it could
    /// not be started, and then children start as they always did.
    static SPAWNER: OnceLock<Option<mpsc::Sender<Request>>> = OnceLock::new();

    pub(super) fn spawn(command: Command) -> io::Result<Child> {
        let Some(spawner) = SPAWNER.get_or_init(start) else {
            return untied(command);
        };
        let (reply, answer) = mpsc::sync_channel(1);
        if let Err(mpsc::SendError((command, _))) = spawner.send((command, reply)) {
            return untied(command);
        }
        answer
            .recv()
            .unwrap_or_else(|_| Err(io::Error::other("the process spawner stopped")))
    }

    /// Never tied: started from any thread but the spawner, a child would be
    /// killed with whichever thread happened to start it.
    fn untied(mut command: Command) -> io::Result<Child> {
        command.spawn()
    }

    fn start() -> Option<mpsc::Sender<Request>> {
        let (sender, requests) = mpsc::channel::<Request>();
        std::thread::Builder::new()
            .name("intern-spawner".into())
            .spawn(move || {
                for (mut command, reply) in requests {
                    tie(&mut command);
                    let _ = reply.send(command.spawn());
                }
            })
            .ok()
            .map(|_| sender)
    }

    fn tie(command: &mut Command) {
        let parent = std::process::id();
        // SAFETY: the hook runs in the child between fork and exec, where
        // only async-signal-safe calls are allowed. prctl, getppid, and nice
        // are, and nothing here allocates or takes a lock.
        unsafe {
            command.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) == -1 {
                    return Err(io::Error::last_os_error());
                }
                // This process can end between the fork and the prctl, and
                // then no signal is coming: the child has been handed to
                // another parent, and must not run on without this one.
                if libc::getppid().unsigned_abs() != parent {
                    return Err(io::Error::from_raw_os_error(libc::ESRCH));
                }
                // Raising niceness needs no privilege, and a failure only
                // leaves the child at Intern's own priority.
                libc::nice(super::SIDECAR_NICENESS);
                Ok(())
            });
        }
    }
}

// Three Win32 calls with no safe wrapper anywhere in the dependency tree. The
// rest of the crate keeps `#![deny(unsafe_code)]`.
#[cfg(windows)]
#[allow(unsafe_code)]
mod windows_job {
    use std::{os::windows::io::AsRawHandle, process::Child, ptr, sync::OnceLock};

    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        },
    };

    /// The one job every child joins, kept for the life of the process and
    /// deliberately never closed: closing it is precisely what kills the
    /// children, so only this process ending may do it. A handle is stored as
    /// a `usize` because a raw pointer is not `Sync`; zero means the job could
    /// not be set up and there is nothing to assign anyone to.
    static JOB: OnceLock<usize> = OnceLock::new();

    pub(super) fn assign(child: &Child) {
        let job = *JOB.get_or_init(create);
        if job == 0 {
            return;
        }
        // SAFETY: `job` is a handle to a job this process owns and never
        // closes, and the child's handle is owned by `child` for the whole
        // call. A failure here is reported through the return value we ignore,
        // not through anything that could be unsound.
        unsafe {
            AssignProcessToJobObject(job as HANDLE, child.as_raw_handle() as HANDLE);
        }
    }

    fn create() -> usize {
        // SAFETY: an unnamed job object with default security, then one call
        // to set its limits with a correctly sized value of exactly the type
        // the information class names. Both are checked for failure.
        unsafe {
            let job = CreateJobObjectW(ptr::null(), ptr::null());
            if job.is_null() {
                return 0;
            }
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let configured = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                ptr::from_ref(&limits).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            if configured == 0 {
                CloseHandle(job);
                return 0;
            }
            job as usize
        }
    }
}

/// The Linux half of what `server::tests::a_launched_child_dies_with_its_parent`
/// proves on Windows, and the thread trap that half has to avoid.
#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::{
        path::Path,
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant},
    };

    use super::{SIDECAR_NICENESS, spawn_tied};

    const CHILD_ANNOUNCEMENT: &str = "INTERN_TEST_CHILD_ANNOUNCEMENT";
    /// How the helper parent ends: `exit` through `std::process::exit`, or
    /// `killed` by a `SIGKILL` from the test.
    const PARENT_ENDS: &str = "INTERN_TEST_PARENT_ENDS";
    const ABRUPT_EXIT_CODE: i32 = 37;

    /// A shell that writes its process id to `announcement`, then becomes a
    /// long sleep under the same id.
    fn announcing_child(announcement: &Path) -> Command {
        let mut command = Command::new("/bin/sh");
        command
            .args([
                "-c",
                r#"echo $$ > "$1.part" && mv "$1.part" "$1" && exec sleep 120"#,
                "sh",
            ])
            .arg(announcement)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command
    }

    fn announced_pid(announcement: &Path) -> Option<u32> {
        std::fs::read_to_string(announcement)
            .ok()?
            .trim()
            .parse()
            .ok()
    }

    fn await_announcement(announcement: &Path) -> u32 {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(pid) = announced_pid(announcement) {
                return pid;
            }
            assert!(
                Instant::now() < deadline,
                "the child should have announced itself"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    /// The fields of `/proc/<pid>/stat` after the parenthesised command
    /// name, from the state on; `None` once the process is gone.
    fn stat_fields(pid: u32) -> Option<Vec<String>> {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let (_, rest) = stat.rsplit_once(')')?;
        Some(rest.split_whitespace().map(str::to_owned).collect())
    }

    /// Alive: neither gone nor a zombie waiting for whoever adopted it.
    fn is_running(pid: u32) -> bool {
        stat_fields(pid).is_some_and(|fields| !matches!(fields[0].as_str(), "Z" | "X"))
    }

    fn niceness(pid: u32) -> i32 {
        stat_fields(pid).expect("a running process")[16]
            .parse()
            .expect("a niceness")
    }

    fn stop(pid: u32) {
        let _ = Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .status();
    }

    fn assert_stops(child: u32) {
        let deadline = Instant::now() + Duration::from_secs(30);
        while is_running(child) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(50));
        }
        let survived = is_running(child);
        if survived {
            stop(child);
        }
        assert!(
            !survived,
            "process {child} outlived the parent that launched it"
        );
    }

    /// Runs the helper below as a parent of its own, which starts a tied
    /// child and ends the way `ends` says, and returns the child's id.
    fn child_of_a_parent_that(ends: &str) -> u32 {
        let workspace = tempfile::tempdir().expect("a temporary directory");
        let announcement = workspace.path().join("child.pid");
        let mut parent = Command::new(std::env::current_exe().expect("the test binary"))
            .args([
                "process::tests::the_parent_of_a_tied_child",
                "--exact",
                "--ignored",
            ])
            .env(CHILD_ANNOUNCEMENT, &announcement)
            .env(PARENT_ENDS, ends)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the helper parent should run");
        let child = await_announcement(&announcement);
        if ends == "killed" {
            parent.kill().expect("the helper parent should be killed");
            let _ = parent.wait();
        } else {
            let status = parent.wait().expect("the helper parent should end");
            assert_eq!(
                status.code(),
                Some(ABRUPT_EXIT_CODE),
                "the helper parent should have left through process::exit"
            );
        }
        child
    }

    /// The point of `PR_SET_PDEATHSIG`: a child of a process that ends
    /// without running a single destructor - through `std::process::exit`,
    /// as the window close and the tray's Quit leave, or killed outright -
    /// must end too.
    #[test]
    fn a_tied_child_dies_with_its_parent_however_the_parent_ends() {
        assert_stops(child_of_a_parent_that("exit"));
        assert_stops(child_of_a_parent_that("killed"));
    }

    /// The trap the spawner thread exists for: the parent the kernel watches
    /// is the thread that started the child. A model started from a thread
    /// that has since finished must keep running.
    #[test]
    fn a_tied_child_outlives_the_thread_that_asked_for_it() {
        let workspace = tempfile::tempdir().expect("a temporary directory");
        let announcement = workspace.path().join("child.pid");
        let request = announcement.clone();
        let mut child = thread::spawn(move || spawn_tied(announcing_child(&request)))
            .join()
            .expect("the asking thread should finish")
            .expect("the child should start");
        let pid = await_announcement(&announcement);
        thread::sleep(Duration::from_millis(500));
        let alive = is_running(pid);
        let _ = child.kill();
        let _ = child.wait();
        assert!(alive, "the child died with the thread that asked for it");
    }

    /// Sidecars run below Intern, as Windows' below-normal class makes them.
    #[test]
    fn a_tied_child_runs_below_this_process() {
        let workspace = tempfile::tempdir().expect("a temporary directory");
        let announcement = workspace.path().join("child.pid");
        let mut child =
            spawn_tied(announcing_child(&announcement)).expect("the child should start");
        let pid = await_announcement(&announcement);
        let expected = (niceness(std::process::id()) + SIDECAR_NICENESS).min(19);
        let actual = niceness(pid);
        let _ = child.kill();
        let _ = child.wait();
        assert_eq!(actual, expected);
    }

    /// The parent in the first test above. Ignored so an ordinary run never
    /// executes it, and inert without the announcement path even then.
    #[test]
    #[ignore = "helper process for a_tied_child_dies_with_its_parent_however_the_parent_ends"]
    fn the_parent_of_a_tied_child() {
        let Some(announcement) = std::env::var_os(CHILD_ANNOUNCEMENT) else {
            return;
        };
        let announcement = Path::new(&announcement);
        let _child = spawn_tied(announcing_child(announcement)).expect("the child should start");
        let _ = await_announcement(announcement);
        if std::env::var(PARENT_ENDS).as_deref() == Ok("killed") {
            thread::sleep(Duration::from_secs(120));
        }
        // No unwinding, no destructors, no stop() - the way Intern's own exit
        // paths leave.
        std::process::exit(ABRUPT_EXIT_CODE);
    }
}
