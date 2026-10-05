#![forbid(unsafe_code)]

use tauri::Manager;

pub mod commands;
pub mod intake;
pub mod microsoft_intake;
pub mod model;
pub mod onboarding;
pub mod secrets;
pub mod sharepoint_root_verifier;
pub mod sharepoint_setup;
pub mod startup;
pub mod tray;

pub fn run() {
    // First, so that even a panic inside Tauri's own setup leaves a record.
    startup::install_panic_hook();
    tauri::Builder::default()
        // One Intern per machine, and registered first as the plugin
        // requires. Autostart at sign-in followed by a click on the shortcut
        // otherwise runs two local models, two intake watchers, and two trays
        // against one queue database.
        .plugin(tauri_plugin_single_instance::init(
            commands::second_instance_launched,
        ))
        .plugin(tauri_plugin_dialog::init())
        // Opens the published user guide and the two SharePoint support links
        // (the provisioned site and the OneDrive download page) in the system
        // browser. A webview <a target="_blank"> has nowhere to go inside
        // Tauri, and the scope in capabilities/default.json admits only those
        // addresses.
        .plugin(tauri_plugin_opener::init())
        // Autostart entries launch Intern with "--minimized" so a sign-in
        // launch can go straight to the tray (when background mode allows it)
        // instead of opening a window nobody asked for. macOS keeps the
        // default LaunchAgent mechanism.
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg("--minimized")
                .build(),
        )
        // Checking for an update is automatic (at launch and on a timer) as
        // well as user-initiated from Settings; installing one never is - the
        // frontend only ever calls the installer after its own click, and the
        // plugin refuses anything not signed with this build's key regardless
        // of which path found it. Microsoft upload verification and hosted
        // inference are separate opt-in network integrations; see their
        // explicit permissions/privacy notices.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            if let Ok(data) = app.path().app_local_data_dir() {
                startup::set_log_directory(&data);
            }
            // Returning the error made Tauri panic with "Failed to setup app"
            // and the process vanish with nothing said anywhere. A failed start
            // is reported instead, and setup succeeds without AppState: the
            // invoke guard below answers every command, and the dialog exits.
            let state = match commands::AppState::initialize(app.handle()) {
                Ok(state) => state,
                Err(error) => {
                    startup::report_failure(app.handle(), &error.code, &error.message);
                    return Ok(());
                }
            };
            let settings = state.settings_snapshot();
            let data = state.data_dir().to_path_buf();
            app.manage(state);
            app.manage(onboarding::OnboardingStore::new(
                data.join("ui-state.json"),
                sharepoint_setup::PACKAGED_DEPLOYMENT,
            ));
            tray::sync_tray(app.handle(), settings.run_in_background);
            // Lossy rather than `args()`, which panics on an argument that is
            // not Unicode; such a path cannot be found anyway.
            let arguments = std::env::args_os()
                .map(|argument| argument.to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            let minimized_launch = arguments.iter().any(|argument| argument == "--minimized");
            let shows_window = startup::shows_window_after_setup(startup::Startup::Ready {
                starts_hidden: tray::window_starts_hidden(
                    settings.start_minimized,
                    settings.run_in_background,
                    minimized_launch,
                ),
                documents: commands::launch_names_documents(&arguments),
            });
            app.resources_table()
                .add(ShutdownGuard(app.handle().clone()));
            // "Send to > Intern" with Intern not yet running starts it with the
            // documents as arguments.
            commands::queue_launch_documents(
                app.handle(),
                arguments,
                std::env::current_dir().unwrap_or_default(),
            );
            if shows_window {
                tray::show_main_window(app.handle());
            }
            Ok(())
        })
        // Close-to-tray. When background mode is on the close request is
        // prevented and the window merely hidden - no teardown of any kind
        // begins, so the deliberate close-time exit behavior for the normal
        // case is left completely alone: when background mode is off (or the
        // settings cannot be read) nothing here touches the event and the
        // window closes exactly as it always has.
        //
        // There is no AppState after a failed start, and then the close is
        // the ordinary one: hiding a window whose app cannot run would strand
        // it.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && window.label() == "main"
                && window
                    .try_state::<commands::AppState>()
                    .is_some_and(|state| state.hide_window_on_close())
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(refused_without_app_state(tauri::generate_handler![
            onboarding::onboarding_status,
            onboarding::onboarding_complete,
            sharepoint_setup::onboarding_sharepoint_status,
            sharepoint_setup::onboarding_start_sharepoint_sync,
            sharepoint_setup::onboarding_activate,
            microsoft_intake::microsoft_intake_status,
            microsoft_intake::microsoft_sign_in_start,
            microsoft_intake::microsoft_sign_in_poll,
            microsoft_intake::microsoft_disconnect,
            microsoft_intake::microsoft_bind_intake,
            microsoft_intake::microsoft_open_sign_in,
            commands::queue_list,
            commands::queue_add_files,
            commands::queue_add_folder,
            commands::queue_pause,
            commands::queue_resume,
            commands::queue_cancel,
            commands::queue_retry,
            commands::queue_remove,
            commands::proposal_approve,
            commands::proposal_keep_original,
            commands::operation_undo,
            commands::settings_get,
            commands::settings_save,
            commands::setup_get,
            commands::setup_start,
            commands::setup_cancel,
            commands::setup_choose_existing,
            commands::history_clear,
            commands::history_list,
            commands::history_export,
            commands::queue_discard_waiting,
            commands::intake_status,
            commands::intake_scan_now,
            commands::folder_classify,
            commands::cloud_roots,
            commands::intake_folder_documents,
            commands::filed_folder_create,
            commands::inbox_folder_create,
            commands::onedrive_open,
            commands::descriptions_status,
            commands::descriptions_backfill,
            commands::hosted_model_status,
            commands::hosted_model_set_key,
            commands::hosted_model_clear_key,
            commands::hosted_model_test,
            commands::house_rules_list,
            commands::house_rule_forget,
            commands::house_rule_use,
        ]))
        .build(tauri::generate_context!())
        .expect("error while running Intern")
        // Every ordinary way out of the app arrives here, and the pipeline is
        // still whole at this point, so llama-server and the parser worker are
        // stopped deliberately rather than left to the kernel. The job object
        // in intern-engine remains the backstop for the exits that never reach
        // this callback: a panic, a crash, and the updater's installer.
        .run(|app, event| {
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) {
                commands::shutdown_runtime(app);
            }
        });
}

/// Every command of Intern's own needs the state a failed start never
/// managed - some through `State`, which would refuse with Tauri's own
/// wording, and some by looking it up, which panics. After a failed start
/// they are all answered APP_NOT_READY instead, while the dialog saying why
/// is on screen. Plugin commands (the dialog, the updater) are not routed
/// through here and keep working.
fn refused_without_app_state<R: tauri::Runtime>(
    commands: impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static,
) -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
    move |invoke| {
        if invoke
            .message
            .webview_ref()
            .try_state::<commands::AppState>()
            .is_none()
        {
            invoke.resolver.reject(commands::app_not_ready());
            return true;
        }
        commands(invoke)
    }
}

/// Parked in the app's resource table for the sake of its `Drop`.
///
/// The updater's install step is the one exit nothing else here sees: it runs
/// the plugin's before-exit hook, hands the installer to the shell, and leaves
/// through `std::process::exit`. That hook is Tauri's `cleanup_before_exit`,
/// which clears this table - so dropping from it is the notice we get, and it
/// arrives while there is still time to release the binaries NSIS must
/// replace.
struct ShutdownGuard(tauri::AppHandle);

impl tauri::Resource for ShutdownGuard {}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        commands::shutdown_runtime(&self.0);
    }
}
