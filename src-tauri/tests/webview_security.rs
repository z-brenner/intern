//! What the webview is allowed to do.
//!
//! The window renders a bundled page and talks to Rust; it has no reason to
//! load a script, a stylesheet, or a frame from anywhere else, and saying so
//! is what stops one injected string from becoming a page that can call every
//! command Intern exposes.

use std::path::Path;

fn config() -> serde_json::Value {
    let config =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json"))
            .expect("the Tauri configuration is readable");
    serde_json::from_str::<serde_json::Value>(&config).expect("the Tauri configuration is JSON")
}

fn security() -> serde_json::Value {
    config()["app"]["security"].clone()
}

#[test]
fn csp_is_configured() {
    let csp = security()["csp"]
        .as_str()
        .expect("the webview must have a content security policy")
        .to_owned();
    for directive in [
        // Nothing loads from anywhere but the bundle.
        "default-src 'self'",
        // The one that matters: no injected or remote script runs.
        "script-src 'self'",
        "object-src 'none'",
        // The window is not framed, and frames nothing.
        "frame-src 'none'",
        "frame-ancestors 'none'",
        // Tauri's own channel, which the commands arrive on.
        "ipc:",
    ] {
        assert!(
            csp.contains(directive),
            "the policy must carry {directive}: {csp}"
        );
    }
}

/// The document viewer shows a picture the window made from the bytes the
/// backend read (a blob address), and draws a PDF with the bundled pdf.js:
/// its worker is one of the app's own scripts, and nothing it needs is
/// evaluated from a string. So the only widening is pictures from blob
/// addresses - no frame, no eval, no worker or script from anywhere else.
#[test]
fn the_document_viewer_widens_only_pictures() {
    let csp = security()["csp"].as_str().unwrap_or_default().to_owned();
    let directive = |name: &str| {
        csp.split(';')
            .map(str::trim)
            .find(|directive| directive.starts_with(&format!("{name} ")))
            .map(str::to_owned)
    };
    assert_eq!(
        directive("img-src").as_deref(),
        Some("img-src 'self' data: blob:")
    );
    assert_eq!(
        directive("script-src").as_deref(),
        Some("script-src 'self'")
    );
    assert_eq!(directive("worker-src"), None, "workers follow script-src");
    assert!(!csp.contains("unsafe-eval"), "{csp}");
}

/// Created visible, the window appeared before setup ran and sat blank and
/// unresponsive through initialization - a white window flashing at sign-in
/// for a launch meant for the tray. Setup shows it as its last step instead
/// (lib.rs, startup::shows_window_after_setup).
#[test]
fn main_window_starts_invisible() {
    let window = &config()["app"]["windows"][0];
    assert_eq!(window["title"], "ATLAS", "the first window is the main one");
    assert_eq!(window["visible"], serde_json::Value::Bool(false));
}
