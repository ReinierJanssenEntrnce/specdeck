mod host;
mod menu;
mod settings;
mod terminal;
mod viewer;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use serde::Serialize;
use specdeck_core::agent;
use specdeck_core::environment::{Environment, PROBE_TIMEOUT};
use specdeck_core::mirror::Colors;
use specdeck_core::peer::PeerConfig;
use specdeck_protocol::AgentId;
use specdeck_protocol::invite::InviteCode;
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::menu::MenuEvent;
use tauri::{
    AppHandle, Emitter, Manager, RunEvent, State, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::host::{HostEvent, HostRuntime, StartOptions};
use crate::settings::Settings;
use crate::viewer::{ViewEvent, ViewerRuntime};

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
        platform: std::env::consts::OS,
    }
}

#[tauri::command]
async fn get_profile(state: State<'_, AppState>) -> Result<Profile, String> {
    Ok(profile(&state))
}

#[tauri::command]
async fn set_username(state: State<'_, AppState>, username: String) -> Result<Profile, String> {
    {
        let mut settings = state.settings.lock().unwrap();
        settings.set_username(&username);
        state.save(&settings);
    }
    Ok(profile(&state))
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
    let runtime = HostRuntime::start(options, &state.environment()).await?;
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
async fn host_close(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(host) = state.host.lock().unwrap().take() {
        host.close();
    }
    refresh_menu(&app);
    Ok(())
}

#[tauri::command]
async fn join_session(
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
    match std::env::var("SPECDECK_PROFILE").ok().filter(|_| cfg!(debug_assertions)) {
        Some(profile) if !profile.is_empty() && profile != "default" => {
            base.join("profiles").join(profile)
        }
        _ => base,
    }
}

fn open_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title(window_title())
        .inner_size(1200.0, 800.0)
        .min_inner_size(720.0, 480.0)
        .build();
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
    if id == menu::ids::QUIT {
        return quit(app);
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
                if let Some(host) = handle.state::<AppState>().host.lock().unwrap().take() {
                    host.close();
                }
                handle.exit(0);
            }
        });
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
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            });
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(&window_title());
            }
            refresh_menu(app.handle());
            Ok(())
        })
        .on_menu_event(on_menu_event)
        .on_window_event(|window, event| {
            if let WindowEvent::Destroyed = event {
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
        ])
        .build(tauri::generate_context!())
        .expect("error while building SpecDeck");

    app.run(|app, event| match event {
        // Closing the last window while hosting keeps the session (and the app) running.
        RunEvent::ExitRequested { api, code: None, .. } => {
            if app.state::<AppState>().host.lock().unwrap().is_some() {
                api.prevent_exit();
            }
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => open_main_window(app),
        _ => {}
    });
}
