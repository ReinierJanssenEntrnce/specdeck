mod host;
mod menu;
mod notify;
mod settings;
mod terminal;
mod tray;
mod viewer;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde::Serialize;
use specdeck_core::agent;
use specdeck_core::environment::{Environment, PROBE_TIMEOUT};
use specdeck_core::mirror::Colors;
use specdeck_core::peer::PeerConfig;
use specdeck_protocol::invite::InviteCode;
use specdeck_protocol::{AgentId, AgentStatus};
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::menu::MenuEvent;
use tauri::{
    AppHandle, Emitter, Manager, RunEvent, State, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_window_state::StateFlags;

use crate::host::{HostEvent, HostNotice, HostRuntime, SharingState, StartOptions, Summary};
use crate::settings::Settings;
use crate::viewer::{EndReason, ViewEvent, ViewerRuntime};

/// Signalling service used when `SPECDECK_SIGNALLING_URL` is not set at build or run time: the
/// local stack from `just local-up`, reached through `just local-dev`'s port-forward.
const DEFAULT_SIGNALLING_URL: &str = "ws://127.0.0.1:18081/signal";

struct AppState {
    settings_path: PathBuf,
    settings: Mutex<Settings>,
    data_dir: PathBuf,
    signalling_url: String,
    /// The user's login-shell environment, read once in the background at start.
    environment: Arc<OnceLock<Environment>>,
    host: Mutex<Option<HostRuntime>>,
    viewer: Mutex<Option<ViewerRuntime>>,
    /// The main window shows the desktop through it (macOS vibrancy, Windows 11 Mica).
    translucent: AtomicBool,
    /// A join link the window has not handled yet.
    join_link: Mutex<Option<JoinLink>>,
}

impl AppState {
    fn environment(&self) -> Environment {
        self.environment.get().cloned().unwrap_or_else(|| Environment::detect(PROBE_TIMEOUT))
    }

    fn save(&self, settings: &Settings) {
        // Failing to remember settings is not worth interrupting the user for.
        let _ = settings.save(&self.settings_path);
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentInfo {
    id: AgentId,
    name: &'static str,
    installed: bool,
    has_timeline: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RecentFolder {
    path: PathBuf,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    username: Option<String>,
    recent_folders: Vec<RecentFolder>,
    last_agent: Option<AgentId>,
    agents: Vec<AgentInfo>,
    /// Set while a session is hosted, so a reopened window goes straight back to it.
    hosting: bool,
    notifications: bool,
    /// The window has a material behind it, so the UI may let it show through.
    translucent: bool,
    platform: &'static str,
}

fn profile(state: &AppState) -> Profile {
    let settings = state.settings.lock().unwrap().clone();
    let agents = agent::detect(&state.environment())
        .into_iter()
        .map(|found| AgentInfo {
            id: found.agent,
            name: found.agent.display_name(),
            installed: found.executable.is_some(),
            has_timeline: found.agent.has_timeline(),
        })
        .collect();
    Profile {
        username: settings.username.clone(),
        recent_folders: settings
            .existing_recent()
            .into_iter()
            .map(|path| RecentFolder {
                name: path.file_name().map_or_else(
                    || path.to_string_lossy().into_owned(),
                    |name| name.to_string_lossy().into_owned(),
                ),
                path,
            })
            .collect(),
        last_agent: settings.last_agent,
        agents,
        hosting: state.host.lock().unwrap().is_some(),
        notifications: settings.notifications,
        translucent: state.translucent.load(Ordering::Relaxed),
        platform: std::env::consts::OS,
    }
}

#[tauri::command]
async fn get_profile(state: State<'_, AppState>) -> Result<Profile, String> {
    Ok(profile(&state))
}

#[tauri::command]
async fn set_username(
    app: AppHandle,
    state: State<'_, AppState>,
    username: String,
) -> Result<Profile, String> {
    {
        let mut settings = state.settings.lock().unwrap();
        settings.set_username(&username);
        state.save(&settings);
    }
    settings_changed(&app);
    Ok(profile(&state))
}

#[tauri::command]
async fn set_notifications(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<Profile, String> {
    {
        let mut settings = state.settings.lock().unwrap();
        settings.notifications = enabled;
        state.save(&settings);
    }
    settings_changed(&app);
    Ok(profile(&state))
}

/// Shows a notification, if the user wants them and the window is not in front.
fn notify(app: &AppHandle, title: &str, body: &str) {
    let enabled = app.state::<AppState>().settings.lock().unwrap().notifications;
    notify::notify(app, enabled, title, body);
}

#[tauri::command]
async fn remove_recent_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<Profile, String> {
    {
        let mut settings = state.settings.lock().unwrap();
        settings.remove_recent(&path);
        state.save(&settings);
    }
    refresh_menu(&app);
    Ok(profile(&state))
}

#[tauri::command]
async fn start_hosting(
    app: AppHandle,
    state: State<'_, AppState>,
    agent: AgentId,
    folder: PathBuf,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    if state.host.lock().unwrap().is_some() {
        return Err("A session is already running. End it before starting another.".into());
    }
    let username = state.settings.lock().unwrap().username.clone().ok_or("Set your name first.")?;
    let options = StartOptions {
        agent,
        folder: folder.clone(),
        cols,
        rows,
        username,
        data_dir: state.data_dir.clone(),
        signalling_url: state.signalling_url.clone(),
        peer: PeerConfig::default(),
    };
    // The menu-bar or tray icon shows the session from the start, and follows it.
    let folder_name = folder.file_name().map_or_else(
        || folder.to_string_lossy().into_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let starting = Summary {
        status: AgentStatus::Starting,
        viewers: 0,
        code: None,
        sharing: SharingState::Connecting,
    };
    let _ = tray::show(&app, &folder_name, agent.display_name(), &starting);
    let observed = app.clone();
    let observer: host::Observer = Arc::new(move |notice: HostNotice| match notice {
        HostNotice::Summary(summary) => {
            tray::update(&observed, &folder_name, agent.display_name(), &summary);
        }
        HostNotice::ViewerJoined(name) => {
            notify(
                &observed,
                &format!("{name} joined"),
                &format!("{name} is watching your session."),
            );
        }
        HostNotice::ViewerLeft(name) => {
            notify(&observed, &format!("{name} left"), &format!("{name} stopped watching."));
        }
        HostNotice::AgentWaiting { message } => {
            let body =
                message.unwrap_or_else(|| "It finished and waits for your next prompt.".into());
            notify(&observed, &format!("{} is waiting for you", agent.display_name()), &body);
        }
    });
    let runtime = match HostRuntime::start(options, &state.environment(), observer).await {
        Ok(runtime) => runtime,
        Err(error) => {
            tray::remove(&app);
            return Err(error);
        }
    };
    *state.host.lock().unwrap() = Some(runtime);
    let mut settings = state.settings.lock().unwrap();
    settings.add_recent(folder);
    settings.last_agent = Some(agent);
    state.save(&settings);
    drop(settings);
    refresh_menu(&app);
    Ok(())
}

/// Runs `f` on the hosted session, or fails if there is none.
fn with_host<T>(state: &AppState, f: impl FnOnce(&HostRuntime) -> T) -> Result<T, String> {
    state.host.lock().unwrap().as_ref().map(f).ok_or_else(|| "No session is running.".into())
}

#[tauri::command]
async fn host_attach(
    state: State<'_, AppState>,
    terminal: Channel<InvokeResponseBody>,
    events: Channel<HostEvent>,
) -> Result<(), String> {
    with_host(&state, |host| host.attach(terminal, events))
}

#[tauri::command]
async fn host_input(state: State<'_, AppState>, data: String) -> Result<(), String> {
    with_host(&state, |host| host.input(&data))?
}

#[tauri::command]
async fn host_resize(state: State<'_, AppState>, cols: u16, rows: u16) -> Result<(), String> {
    with_host(&state, |host| host.resize(cols, rows))
}

#[tauri::command]
async fn host_set_colors(
    state: State<'_, AppState>,
    foreground: String,
    background: String,
) -> Result<(), String> {
    let (Some(foreground), Some(background)) = (parse_hex(&foreground), parse_hex(&background))
    else {
        return Err("Colours must be #rrggbb.".into());
    };
    with_host(&state, |host| host.set_colors(Colors { foreground, background }))
}

#[tauri::command]
async fn host_new_code(state: State<'_, AppState>) -> Result<(), String> {
    with_host(&state, HostRuntime::new_code)
}

#[tauri::command]
async fn host_remove_viewer(state: State<'_, AppState>, viewer_id: u32) -> Result<(), String> {
    with_host(&state, |host| host.remove_viewer(viewer_id))
}

#[tauri::command]
async fn host_set_viewer_limit(state: State<'_, AppState>, limit: u8) -> Result<(), String> {
    with_host(&state, |host| host.set_viewer_limit(limit))
}

#[tauri::command]
async fn host_close(app: AppHandle) -> Result<(), String> {
    end_session(&app);
    Ok(())
}

/// Ends the hosted session, if any, and everything that showed it outside the window.
fn end_session(app: &AppHandle) {
    if let Some(host) = app.state::<AppState>().host.lock().unwrap().take() {
        host.close();
    }
    tray::remove(app);
    refresh_menu(app);
}

#[tauri::command]
async fn join_session(
    app: AppHandle,
    state: State<'_, AppState>,
    code: String,
    terminal: Channel<InvokeResponseBody>,
    events: Channel<ViewEvent>,
) -> Result<(), String> {
    let code = InviteCode::from_join_link(code.trim())
        .or_else(|_| InviteCode::parse(&code))
        .map_err(|_| "That is not an invite code. It looks like 7K2M-9QXH-4TPA.".to_string())?;
    let username = state.settings.lock().unwrap().username.clone().ok_or("Set your name first.")?;
    let runtime = ViewerRuntime::join(
        code,
        username,
        state.signalling_url.clone(),
        PeerConfig::default(),
        terminal,
        events,
        move |reason| match reason {
            EndReason::Closed => notify(
                &app,
                "The host ended the session",
                "You can keep reading what was shared until you leave.",
            ),
            EndReason::Removed => {
                notify(&app, "The host removed you", "You are no longer connected to the session.")
            }
            _ => {}
        },
    );
    if let Some(previous) = state.viewer.lock().unwrap().replace(runtime) {
        previous.leave();
    }
    Ok(())
}

#[tauri::command]
async fn leave_session(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(viewer) = state.viewer.lock().unwrap().take() {
        viewer.leave();
    }
    Ok(())
}

fn parse_hex(color: &str) -> Option<specdeck_core::mirror::Rgb> {
    let hex = color.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some(specdeck_core::mirror::Rgb { r: channel(0)?, g: channel(2)?, b: channel(4)? })
}

/// Separate settings and data per profile, so two apps can run side by side for local testing
/// (`just local-dev`). Only debug builds read `SPECDECK_PROFILE`.
fn profile_dir(base: PathBuf) -> PathBuf {
    match test_profile() {
        Some(profile) => base.join("profiles").join(profile),
        None => base,
    }
}

/// The `SPECDECK_PROFILE` of a debug build, unless it is the default one.
fn test_profile() -> Option<String> {
    std::env::var("SPECDECK_PROFILE")
        .ok()
        .filter(|profile| cfg!(debug_assertions) && !profile.is_empty() && profile != "default")
}

/// A join link the window has not handled yet: the code, or why the link does not work.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct JoinLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// A `specdeck://` link was opened: keep it for the window and bring the window to the front.
/// The window takes it with `take_join_link`, at start or when told about it.
fn open_link(app: &AppHandle, link: &str) {
    let parsed = InviteCode::from_join_link(link.trim());
    let pending = match parsed {
        Ok(code) => JoinLink { code: Some(code.to_string()), error: None },
        Err(_) => JoinLink {
            code: None,
            error: Some("That link is not a SpecDeck invite. Ask the host for a new one.".into()),
        },
    };
    *app.state::<AppState>().join_link.lock().unwrap() = Some(pending);
    open_main_window(app);
    let _ = app.emit_to("main", "menu", MenuCommand { id: "join-link".into(), path: None });
}

#[tauri::command]
async fn take_join_link(state: State<'_, AppState>) -> Result<Option<JoinLink>, String> {
    Ok(state.join_link.lock().unwrap().take())
}

fn open_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title(window_title())
        .inner_size(1200.0, 800.0)
        .min_inner_size(720.0, 480.0);
    // macOS: the toolbar is the title bar, with the window buttons centred in it.
    #[cfg(target_os = "macos")]
    {
        builder = builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true)
            .traffic_light_position(tauri::LogicalPosition::new(16.0, 20.0));
    }
    // Transparent only where a material can fill it; Linux stays opaque.
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        builder = builder.transparent(true);
    }
    let Ok(window) = builder.build() else { return };
    let translucent = apply_material(&window);
    app.state::<AppState>().translucent.store(translucent, Ordering::Relaxed);
}

/// Puts the platform's window material behind the page: the sidebar vibrancy on macOS, Mica on
/// Windows 11. Returns whether it worked; without it the page keeps an opaque background.
fn apply_material(window: &tauri::WebviewWindow) -> bool {
    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{NSVisualEffectMaterial, NSVisualEffectState};
        window_vibrancy::apply_vibrancy(
            window,
            NSVisualEffectMaterial::Sidebar,
            Some(NSVisualEffectState::FollowsWindowActiveState),
            None,
        )
        .is_ok()
    }
    #[cfg(target_os = "windows")]
    {
        window_vibrancy::apply_mica(window, None).is_ok()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = window;
        false
    }
}

/// Rebuilds the menu from the recent folders and whether a session is hosted.
fn refresh_menu(app: &AppHandle) {
    let state = app.state::<AppState>();
    let recent = state.settings.lock().unwrap().existing_recent();
    let hosting = state.host.lock().unwrap().is_some();
    if let Ok(menu) = menu::build(app, &menu::MenuState { recent: &recent, hosting }) {
        let _ = app.set_menu(menu);
    }
}

/// A menu command for the UI.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MenuCommand {
    id: String,
    /// The folder, for a recent folder.
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    match id {
        menu::ids::QUIT => return quit(app),
        menu::ids::SETTINGS => return open_settings_window(app),
        tray::ids::OPEN => return open_main_window(app),
        tray::ids::COPY_CODE => return copy_invite_code(app),
        tray::ids::END => return confirm_end_session(app),
        _ => {}
    }
    let command = match id.strip_prefix(menu::RECENT_PREFIX) {
        Some(index) => {
            let recent = app.state::<AppState>().settings.lock().unwrap().existing_recent();
            let Some(path) = index.parse::<usize>().ok().and_then(|i| recent.get(i).cloned())
            else {
                return;
            };
            MenuCommand { id: "recent".into(), path: Some(path) }
        }
        None => MenuCommand { id: id.into(), path: None },
    };
    // The menu works with the window closed too (macOS, while hosting).
    open_main_window(app);
    let _ = app.emit("menu", command);
}

/// Copies the invite code from the tray, which works with the window closed too.
fn copy_invite_code(app: &AppHandle) {
    let code = app.state::<AppState>().host.lock().unwrap().as_ref().and_then(HostRuntime::code);
    if let Some(code) = code {
        let _ = app.clipboard().write_text(code);
    }
}

/// Ends the session from the tray, after the same question the window asks.
fn confirm_end_session(app: &AppHandle) {
    let handle = app.clone();
    app.dialog()
        .message("Everyone watching is disconnected, the agent stops, and the invite code stops working.")
        .title("End the session for everyone?")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom("End Session".into(), "Cancel".into()))
        .show(move |confirmed| {
            if confirmed {
                end_session(&handle);
                // An open window goes back to the start screen.
                let _ = handle.emit("menu", MenuCommand { id: "session-ended".into(), path: None });
            }
        });
}

/// Quits, after confirming when that would end a hosted session for everyone.
fn quit(app: &AppHandle) {
    if app.state::<AppState>().host.lock().unwrap().is_none() {
        return app.exit(0);
    }
    let handle = app.clone();
    app.dialog()
        .message(
            "Quitting ends your session. Everyone watching is disconnected and the agent stops.",
        )
        .title("End the session and quit?")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "End Session and Quit".into(),
            "Cancel".into(),
        ))
        .show(move |confirmed| {
            if confirmed {
                end_session(&handle);
                handle.exit(0);
            }
        });
}

/// The settings window: one at a time, brought to the front when open.
fn open_settings_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }
    let title = if cfg!(target_os = "macos") { "Settings" } else { "SpecDeck Settings" };
    let _ = WebviewWindowBuilder::new(
        app,
        "settings",
        WebviewUrl::App("index.html?window=settings".into()),
    )
    .title(title)
    .inner_size(460.0, 300.0)
    .resizable(false)
    .minimizable(false)
    .maximizable(false)
    .center()
    .build();
}

/// Tells the main window that the settings changed, so it shows the new name.
fn settings_changed(app: &AppHandle) {
    let _ = app.emit_to("main", "menu", MenuCommand { id: "profile-changed".into(), path: None });
}

fn window_title() -> String {
    match std::env::var("SPECDECK_PROFILE").ok().filter(|p| cfg!(debug_assertions) && !p.is_empty())
    {
        Some(profile) => format!("SpecDeck ({profile})"),
        None => "SpecDeck".into(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    // One instance per user: starting the app again, or a join link on Windows and Linux, goes
    // to the running one (links through the deep-link plugin). The test profiles of
    // `just local-dev` run side by side instead.
    if test_profile().is_none() {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            open_main_window(app);
        }));
    }
    // Window size and position per profile; the settings window always opens centred.
    let window_state_file = match test_profile() {
        Some(profile) => format!(".window-state-{profile}.json"),
        None => ".window-state.json".into(),
    };
    let app = builder
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_filename(window_state_file)
                .with_denylist(&["settings"])
                .with_state_flags(StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED)
                .build(),
        )
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let config_dir = profile_dir(app.path().app_config_dir()?);
            let data_dir = profile_dir(app.path().app_cache_dir()?).join("sessions");
            let settings_path = config_dir.join("settings.json");
            let environment = Arc::new(OnceLock::new());
            let background = environment.clone();
            std::thread::spawn(move || {
                let _ = background.set(Environment::detect(PROBE_TIMEOUT));
            });
            let signalling_url = std::env::var("SPECDECK_SIGNALLING_URL")
                .ok()
                .or_else(|| option_env!("SPECDECK_SIGNALLING_URL").map(str::to_owned))
                .unwrap_or_else(|| DEFAULT_SIGNALLING_URL.to_owned());
            app.manage(AppState {
                settings: Mutex::new(Settings::load(&settings_path)),
                settings_path,
                data_dir,
                signalling_url,
                environment,
                host: Mutex::new(None),
                viewer: Mutex::new(None),
                translucent: AtomicBool::new(false),
                join_link: Mutex::new(None),
            });
            open_main_window(app.handle());
            refresh_menu(app.handle());
            // Join links: the one the app was started with, and any opened while it runs.
            #[cfg(any(windows, target_os = "linux"))]
            let _ = app.deep_link().register_all();
            if let Ok(Some(urls)) = app.deep_link().get_current()
                && let Some(url) = urls.first()
            {
                open_link(app.handle(), url.as_str());
            }
            let links = app.handle().clone();
            app.deep_link().on_open_url(move |event| {
                if let Some(url) = event.urls().first() {
                    open_link(&links, url.as_str());
                }
            });
            // A click on a notification brings the session to the front.
            let handle = app.handle().clone();
            let _ = app.notification().on_action(move |_| open_main_window(&handle));
            Ok(())
        })
        .on_menu_event(on_menu_event)
        .on_window_event(|window, event| {
            // Only the main window shows a session; the settings window comes and goes.
            if let WindowEvent::Destroyed = event
                && window.label() == "main"
            {
                let state = window.state::<AppState>();
                // A hosted session keeps running without its window; viewing ends with it.
                if let Some(host) = state.host.lock().unwrap().as_ref() {
                    host.detach();
                }
                if let Some(viewer) = state.viewer.lock().unwrap().take() {
                    viewer.leave();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_profile,
            set_username,
            set_notifications,
            remove_recent_folder,
            start_hosting,
            host_attach,
            host_input,
            host_resize,
            host_set_colors,
            host_new_code,
            host_remove_viewer,
            host_set_viewer_limit,
            host_close,
            join_session,
            leave_session,
            take_join_link,
        ])
        .build(tauri::generate_context!())
        .expect("error while building SpecDeck");

    app.run(|app, event| match event {
        // Closing the last window while hosting keeps the session (and the app) running.
        RunEvent::ExitRequested { api, code: None, .. }
            if app.state::<AppState>().host.lock().unwrap().is_some() =>
        {
            api.prevent_exit();
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => open_main_window(app),
        _ => {}
    });
}
