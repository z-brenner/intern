//! What the worker says when it panics.
//!
//! The worker's standard error is kept in a log file beside the model
//! server's, for support to read. Rust's default panic hook prints the panic
//! message there, and a message can quote the document: slicing a string at a
//! bad index prints the string, and a parser's own panic can carry whatever
//! it was reading. So the worker replaces the hook with one that reports
//! where the panic happened and how large its message was - enough to find
//! the line - and never a byte of the message itself.

use std::{any::Any, io::Write as _, panic::PanicHookInfo};

/// Replaces the panic hook for the rest of the process. Call it before
/// anything else can panic.
pub fn install() {
    std::panic::set_hook(Box::new(|info: &PanicHookInfo<'_>| {
        let line = report(
            info.location()
                .map(|location| (location.file(), location.line())),
            info.payload(),
        );
        let _ = writeln!(std::io::stderr().lock(), "{line}");
    }));
}

/// One JSON line for a panic: its source location, and the type and byte
/// length of its payload. The payload's contents are never read past their
/// length.
pub fn report(location: Option<(&str, u32)>, payload: &(dyn Any + Send)) -> String {
    let (kind, bytes) = if let Some(message) = payload.downcast_ref::<&'static str>() {
        ("str", message.len())
    } else if let Some(message) = payload.downcast_ref::<String>() {
        ("String", message.len())
    } else {
        ("other", 0)
    };
    let location = location.map_or_else(
        || "unknown".to_owned(),
        |(file, line)| format!("{file}:{line}"),
    );
    format!(
        "{{\"level\":\"error\",\"code\":\"WORKER_PANIC\",\"location\":{},\"payload\":\"{kind}\",\"payload_bytes\":{bytes}}}",
        serde_json::to_string(&location).unwrap_or_else(|_| "\"unknown\"".to_owned())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "CONFIDENTIAL-TEXT";
    const CHILD: &str = "INTERN_TEST_PANIC_HOOK_CHILD";

    #[test]
    fn a_report_names_the_place_and_never_the_message() {
        let owned: Box<dyn Any + Send> =
            Box::new(format!("byte index 40 is out of range of `{SECRET}`"));
        let line = report(
            Some(("crates/intern-worker/src/email.rs", 453)),
            owned.as_ref(),
        );
        assert!(!line.contains(SECRET), "{line}");
        let parsed: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(parsed["code"], "WORKER_PANIC");
        assert_eq!(parsed["location"], "crates/intern-worker/src/email.rs:453");
        assert_eq!(parsed["payload"], "String");
        assert_eq!(parsed["payload_bytes"], 52);

        let fixed: Box<dyn Any + Send> = Box::new(SECRET);
        let line = report(None, fixed.as_ref());
        assert!(!line.contains(SECRET), "{line}");
        assert!(line.contains("\"payload\":\"str\""), "{line}");
        assert!(line.contains("\"location\":\"unknown\""), "{line}");

        let other: Box<dyn Any + Send> = Box::new(17_u32);
        assert!(report(None, other.as_ref()).contains("\"payload\":\"other\""));
    }

    /// The real thing: a string sliced past its end, the panic whose default
    /// message quotes the string, in a process with the hook installed. The
    /// panic happens in a child, because the hook is process-wide.
    #[test]
    fn a_panic_over_document_text_leaves_no_trace_of_it() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "panic_hook::tests::panics_over_document_text",
                "--exact",
                "--ignored",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{stdout}{stderr}");
        assert!(stdout.contains("1 passed"), "{stdout}");
        assert!(stderr.contains("\"code\":\"WORKER_PANIC\""), "{stderr}");
        assert!(stderr.contains("panic_hook.rs:"), "{stderr}");
        assert!(!stdout.contains("CONFIDENTIAL"), "{stdout}");
        assert!(!stderr.contains("CONFIDENTIAL"), "{stderr}");
    }

    /// The child half of the test above; inert unless that test started it.
    #[test]
    #[ignore = "child process of a_panic_over_document_text_leaves_no_trace_of_it"]
    fn panics_over_document_text() {
        if std::env::var_os(CHILD).is_none() {
            return;
        }
        install();
        let text = format!("{SECRET} from page one");
        let end = std::hint::black_box(text.len() + 40);
        let caught = std::panic::catch_unwind(|| text[..end].len());
        assert!(caught.is_err());
    }
}
