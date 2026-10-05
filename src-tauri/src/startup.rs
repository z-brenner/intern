//! What Intern leaves behind when it cannot start or when it crashes, and
//! when its window appears.
//!
//! A release build has no console (`windows_subsystem = "windows"`), so a
//! panic message and every `eprintln!` in the shell went nowhere. A corrupt
//! queue database made Intern flash a window and vanish on every launch, with
//! no error code, no message, and no file for the user or for support. Now a
//! panic leaves one line in `logs/intern.log`, and a startup failure is shown
//! in a dialog and written to `logs/startup-error.log`, both under the app's
//! local data folder (the temp folder when that folder is the problem).
//!
//! Neither log carries document text. A startup failure is one of Intern's own
//! codes and messages; a panic is reduced to where it happened and how long its
//! message was, because a panic message can quote whatever value was in hand -
//! a line of a contract, a party's name - and a log file outlives the session.

use std::{
    any::Any,
    fs,
    io::Write,
    panic::Location,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

/// Where panics are recorded, and what happened with no window to say it in.
pub const PANIC_LOG: &str = "intern.log";
/// Where startup failures are recorded.
pub const STARTUP_ERROR_LOG: &str = "startup-error.log";
/// A log over this size is started again rather than appended to, so a crash
/// at every sign-in for a year cannot fill a disk.
const LOG_LIMIT_BYTES: u64 = 1024 * 1024;

/// The data folder's `logs`, once setup has found it. Panics before then,
/// and on a machine where it cannot be found, go to the temp folder.
static LOG_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

/// Records every panic, from any thread, from here on. Installed first thing
/// in `run`, before Tauri can panic over a setup it could not finish.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let record = panic_record(&timestamp(), thread.name(), info.location(), info.payload());
        // A panic hook must never panic itself, and the data folder can be the
        // very thing that failed: the temp folder is the second chance.
        if append_log_line(&log_directory(), PANIC_LOG, &record).is_err() {
            let _ = append_log_line(&fallback_log_directory(), PANIC_LOG, &record);
        }
        // The default hook prints the message, so only a developer's build
        // keeps it; a release build has nowhere for it to go but a terminal.
        if cfg!(debug_assertions) {
            previous(info);
        }
    }));
}

/// Appends a line to `logs/intern.log` about something that happened with
/// no window to say it in, such as documents a launch named that could not be
/// queued. Codes and counts only, never a document's name or text.
pub fn log_line(message: &str) {
    let record = format!("{} {message}", timestamp());
    if append_log_line(&log_directory(), PANIC_LOG, &record).is_err() {
        let _ = append_log_line(&fallback_log_directory(), PANIC_LOG, &record);
    }
}

/// Points the logs at the app's data folder, as soon as setup knows it.
pub fn set_log_directory(data: &Path) {
    let _ = LOG_DIRECTORY.set(data.join("logs"));
}

fn log_directory() -> PathBuf {
    LOG_DIRECTORY
        .get()
        .cloned()
        .unwrap_or_else(fallback_log_directory)
}

fn fallback_log_directory() -> PathBuf {
    std::env::temp_dir().join("Intern").join("logs")
}

fn timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// One line describing a panic: when, on which thread, and where. Never what
/// it said - only what kind of message it carried and how long it was, which
/// tells two crashes at one line apart without repeating a word of either.
pub(crate) fn panic_record(
    timestamp: &str,
    thread: Option<&str>,
    location: Option<&Location<'_>>,
    payload: &(dyn Any + Send),
) -> String {
    let location = location.map_or_else(
        || "an unknown location".to_owned(),
        |location| {
            format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        },
    );
    let message = if let Some(text) = payload.downcast_ref::<&str>() {
        format!("a {}-byte message, withheld", text.len())
    } else if let Some(text) = payload.downcast_ref::<String>() {
        format!("a {}-byte message, withheld", text.len())
    } else {
        "a payload that is not text".to_owned()
    };
    format!(
        "{timestamp} panic on thread '{}' at {location}: {message}",
        thread.unwrap_or("<unnamed>")
    )
}

/// Appends one line to `directory/file`, creating both as needed, and says
/// where it went.
pub(crate) fn append_log_line(
    directory: &Path,
    file: &str,
    line: &str,
) -> std::io::Result<PathBuf> {
    fs::create_dir_all(directory)?;
    let path = directory.join(file);
    let oversized = fs::metadata(&path).is_ok_and(|metadata| metadata.len() > LOG_LIMIT_BYTES);
    let mut options = fs::OpenOptions::new();
    if oversized {
        options.write(true).create(true).truncate(true);
    } else {
        options.append(true).create(true);
    }
    let mut log = options.open(&path)?;
    writeln!(log, "{line}")?;
    Ok(path)
}

/// Writes why Intern could not start where support will ask for it.
pub(crate) fn record_startup_failure(
    directory: &Path,
    timestamp: &str,
    code: &str,
    message: &str,
) -> std::io::Result<PathBuf> {
    append_log_line(
        directory,
        STARTUP_ERROR_LOG,
        &format!("{timestamp} {code}: {message}"),
    )
}

/// Records a failed start under `data`'s `logs` - or in `fallback` (the temp
/// folder's), when the data folder is what failed or cannot be written - and
/// returns what the dialog says, naming where the record went.
pub(crate) fn record_failure(
    data: Option<&Path>,
    fallback: &Path,
    timestamp: &str,
    code: &str,
    message: &str,
) -> String {
    let directory = data.map_or_else(|| fallback.to_path_buf(), |data| data.join("logs"));
    let log = record_startup_failure(&directory, timestamp, code, message)
        .or_else(|_| record_startup_failure(fallback, timestamp, code, message))
        .ok();
    startup_failure_text(code, message, data, log.as_deref())
}

/// What the startup failure dialog says: what failed, in Intern's own words
/// and code, where Intern keeps its data - the folder a corrupt or locked
/// database lives in - and where the record of it is.
pub(crate) fn startup_failure_text(
    code: &str,
    message: &str,
    data_folder: Option<&Path>,
    log: Option<&Path>,
) -> String {
    let data_folder = data_folder.map_or_else(
        || "unavailable".to_owned(),
        |folder| folder.display().to_string(),
    );
    let mut text = format!(
        "Intern could not start: {}.\n\nError code: {code}\nData folder: {data_folder}",
        message.trim_end_matches('.')
    );
    if let Some(log) = log {
        text.push_str(&format!("\nThis was saved to {}.", log.display()));
    }
    text.push_str("\n\nIf restarting Intern does not help, send this message to support.");
    text
}

/// How setup ended, as far as the window is concerned.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Startup {
    /// Initialization failed and a dialog says why.
    Failed,
    /// Running. `starts_hidden` is `tray::window_starts_hidden`; `documents`
    /// says the launch named documents to add.
    Ready {
        starts_hidden: bool,
        documents: bool,
    },
}

/// Whether setup ends by showing the main window.
///
/// The window is created hidden (tauri.conf.json): created visible, it sat
/// blank and unresponsive through initialization, so a sign-in launch meant
/// for the tray flashed an empty window that could read "Not Responding"
/// before it hid. Showing it is now setup's last act, and this is the only
/// place the choice is made. A failed start always shows it, even for a
/// launch meant for the tray, or a failing launch would be wholly invisible;
/// documents sent to Intern show it, because someone just asked for them.
pub fn shows_window_after_setup(startup: Startup) -> bool {
    match startup {
        Startup::Failed => true,
        Startup::Ready {
            starts_hidden,
            documents,
        } => documents || !starts_hidden,
    }
}

/// Tells the person Intern could not start, and leaves a record of why.
///
/// The dialog is the non-blocking kind: setup runs on the main thread, which
/// the blocking kind would freeze along with the dialog itself. Intern exits
/// with an error when it is dismissed; until then the window stays open
/// behind it and every command answers APP_NOT_READY.
pub fn report_failure(app: &AppHandle, code: &str, message: &str) {
    let data = app.path().app_local_data_dir().ok();
    let text = record_failure(
        data.as_deref(),
        &fallback_log_directory(),
        &timestamp(),
        code,
        message,
    );
    let window = app.get_webview_window("main");
    if shows_window_after_setup(Startup::Failed)
        && let Some(window) = &window
    {
        let _ = window.show();
        let _ = window.set_focus();
    }
    let mut dialog = app
        .dialog()
        .message(text)
        .title("Intern could not start")
        .kind(MessageDialogKind::Error);
    if let Some(window) = &window {
        dialog = dialog.parent(window);
    }
    let exit = app.clone();
    dialog.show(move |_| exit.exit(1));
}

#[cfg(test)]
mod tests {
    use std::panic::Location;

    use super::{
        PANIC_LOG, STARTUP_ERROR_LOG, Startup, append_log_line, panic_record, record_failure,
        record_startup_failure, shows_window_after_setup, startup_failure_text,
    };

    fn scratch(name: &str) -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "intern-startup-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn panic_hook_writes_without_document_text() {
        let directory = scratch("panic");
        let location = Location::caller();
        let secret = String::from("CONFIDENTIAL settlement between Acme Corp and John Smith");
        let owned: Box<dyn std::any::Any + Send> = Box::new(secret.clone());
        let borrowed: Box<dyn std::any::Any + Send> = Box::new("CONFIDENTIAL");

        let record = panic_record(
            "2026-10-05T07:32:56Z",
            Some("intern-pipeline-scheduler"),
            Some(location),
            owned.as_ref(),
        );
        let path = append_log_line(&directory.join("logs"), PANIC_LOG, &record).unwrap();
        append_log_line(
            &directory.join("logs"),
            PANIC_LOG,
            &panic_record("2026-10-05T07:33:00Z", None, None, borrowed.as_ref()),
        )
        .unwrap();

        let log = std::fs::read_to_string(&path).unwrap();
        assert!(!log.contains("CONFIDENTIAL"), "{log}");
        assert!(!log.contains("Acme"), "{log}");
        // Where, when, and on which thread: enough to find the line.
        assert!(log.contains(&format!("{}:{}", location.file(), location.line())));
        assert!(log.contains("2026-10-05T07:32:56Z"));
        assert!(log.contains("'intern-pipeline-scheduler'"));
        assert!(log.contains(&format!("a {}-byte message, withheld", secret.len())));
        // Appended, not replaced: the second panic is the second line.
        assert_eq!(log.lines().count(), 2);
        assert!(
            log.lines()
                .nth(1)
                .unwrap()
                .contains("'<unnamed>' at an unknown location")
        );
        assert!(
            log.lines()
                .nth(1)
                .unwrap()
                .contains("a 12-byte message, withheld")
        );
        let opaque: Box<dyn std::any::Any + Send> = Box::new(42_u32);
        assert!(
            panic_record("t", None, None, opaque.as_ref()).ends_with("a payload that is not text")
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn startup_error_is_logged_and_reported() {
        let directory = scratch("startup");
        let data = directory.join("com.intern.app");
        let message = "the rename history database is unavailable";

        let log = record_startup_failure(
            &data.join("logs"),
            "2026-10-05T07:32:56Z",
            "APP_DATA_UNAVAILABLE",
            message,
        )
        .unwrap();

        assert_eq!(log, data.join("logs").join(STARTUP_ERROR_LOG));
        assert_eq!(
            std::fs::read_to_string(&log).unwrap(),
            format!("2026-10-05T07:32:56Z APP_DATA_UNAVAILABLE: {message}\n")
        );
        let text = startup_failure_text("APP_DATA_UNAVAILABLE", message, Some(&data), Some(&log));
        assert!(text.contains(message));
        assert!(text.contains("Error code: APP_DATA_UNAVAILABLE"));
        assert!(text.contains(&format!("Data folder: {}", data.display())));
        assert!(text.contains(&log.display().to_string()));
        // A data folder that could not be found is said to be unavailable,
        // not left as an empty line.
        assert!(
            startup_failure_text("APP_DATA_UNAVAILABLE", message, None, None)
                .contains("Data folder: unavailable")
        );
        // The failure path shows the window, even for a launch meant for the
        // tray: the dialog has to have something to sit on.
        assert!(shows_window_after_setup(Startup::Failed));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn a_corrupt_queue_database_fails_startup_with_a_logged_and_reported_code() {
        // The failure TAURI_SHELL-6 describes: queue.sqlite3 in the data
        // folder is not a database. Opening it is AppState::initialize's
        // first use of the queue, and setup hands the error it gets to
        // `report_failure`, which records it through `record_failure`.
        let data = scratch("corrupt").join("com.intern.app");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(
            data.join("queue.sqlite3"),
            b"this is not a sqlite database, it is a damaged file",
        )
        .unwrap();

        let error = match intern_core::QueueStore::open(data.join("queue.sqlite3")) {
            Ok(_) => panic!("a damaged database must not open"),
            Err(error) => {
                crate::commands::CommandError::from(intern_queue::PipelineError::from(error))
            }
        };
        let fallback = data.with_file_name("temp-logs");
        let text = record_failure(Some(&data), &fallback, "t", &error.code, &error.message);

        let log = data.join("logs").join(STARTUP_ERROR_LOG);
        let written = std::fs::read_to_string(&log).unwrap();
        assert_eq!(written, format!("t {}: {}\n", error.code, error.message));
        assert_eq!(error.code, "DATABASE_UNAVAILABLE");
        assert!(text.contains("Error code: DATABASE_UNAVAILABLE"), "{text}");
        assert!(text.contains(&format!("Data folder: {}", data.display())));
        assert!(text.contains(&log.display().to_string()));
        assert!(!fallback.exists(), "the data folder took the record");
        std::fs::remove_dir_all(data.parent().unwrap()).unwrap();
    }

    #[test]
    fn a_data_folder_that_cannot_hold_the_log_sends_it_to_the_temp_folder() {
        // The data folder can be the very thing that failed: here a file
        // stands where its `logs` folder would go.
        let root = scratch("unwritable");
        let data = root.join("com.intern.app");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("logs"), b"not a folder").unwrap();
        let fallback = root.join("temp").join("Intern").join("logs");

        let text = record_failure(
            Some(&data),
            &fallback,
            "t",
            "APP_DATA_UNAVAILABLE",
            "the data folder is unavailable",
        );

        let log = fallback.join(STARTUP_ERROR_LOG);
        assert_eq!(
            std::fs::read_to_string(&log).unwrap(),
            "t APP_DATA_UNAVAILABLE: the data folder is unavailable\n"
        );
        assert!(text.contains(&log.display().to_string()), "{text}");
        // Still names the data folder: that is where the trouble is.
        assert!(text.contains(&format!("Data folder: {}", data.display())));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_log_that_grew_past_its_limit_starts_again() {
        let directory = scratch("limit");
        let path = directory.join(PANIC_LOG);
        std::fs::write(&path, vec![b'x'; 1024 * 1024 + 1]).unwrap();

        append_log_line(&directory, PANIC_LOG, "fresh").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "fresh\n");
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn the_window_appears_when_setup_is_done_unless_the_tray_was_asked_for() {
        assert!(shows_window_after_setup(Startup::Ready {
            starts_hidden: false,
            documents: false,
        }));
        // A sign-in launch with background mode on goes straight to the tray.
        assert!(!shows_window_after_setup(Startup::Ready {
            starts_hidden: true,
            documents: false,
        }));
        // Documents sent to Intern are something a person just asked for.
        assert!(shows_window_after_setup(Startup::Ready {
            starts_hidden: true,
            documents: true,
        }));
        assert!(shows_window_after_setup(Startup::Failed));
    }
}
