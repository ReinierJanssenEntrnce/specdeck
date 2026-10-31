//! The native menu (design decision 11): the application menu on macOS, a File, Edit, View and
//! Help menu bar on Windows and Linux.
//!
//! Most items are handled by the UI, which gets their id in a `menu` event. The host's terminal
//! gets first refusal on keys, so on Windows and Linux no item takes a key the terminal needs
//! (Ctrl+C, Ctrl+F, Ctrl+N, Ctrl+W and the like); the UI handles those keys itself.

use std::path::PathBuf;

use tauri::menu::{AboutMetadata, Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Runtime};

/// Menu items whose id starts with this open a recent folder; the rest is its index.
pub const RECENT_PREFIX: &str = "recent:";

/// Ids the UI handles, as sent in the `menu` event.
pub mod ids {
    pub const HOST: &str = "host";
    pub const JOIN: &str = "join";
    pub const COPY_CODE: &str = "copy-code";
    pub const FIND: &str = "find";
    pub const SETTINGS: &str = "settings";
    pub const TIMELINE: &str = "tab:timeline";
    pub const FILES: &str = "tab:files";
    pub const PEOPLE: &str = "tab:people";
    pub const JUMP_TO_LIVE: &str = "jump-to-live";
    pub const SHORTCUTS: &str = "shortcuts";
    /// Handled in Rust: confirms first while a session is hosted.
    pub const QUIT: &str = "quit";
    /// Edit commands on Windows and Linux, where the native ones would take terminal keys.
    pub const COPY: &str = "copy";
    pub const PASTE: &str = "paste";
    pub const SELECT_ALL: &str = "select-all";
}

/// What the menu depends on.
pub struct MenuState<'a> {
    pub recent: &'a [PathBuf],
    pub hosting: bool,
}

/// An accelerator for macOS only, or one for every platform.
fn keys(mac: &'static str, others: Option<&'static str>) -> Option<&'static str> {
    if cfg!(target_os = "macos") { Some(mac) } else { others }
}

fn item<R: Runtime>(
    app: &AppHandle<R>,
    id: &str,
    text: &str,
    accelerator: Option<&str>,
    enabled: bool,
) -> tauri::Result<tauri::menu::MenuItem<R>> {
    let mut builder = MenuItemBuilder::with_id(id, text).enabled(enabled);
    if let Some(accelerator) = accelerator {
        builder = builder.accelerator(accelerator);
    }
    builder.build(app)
}

pub fn build<R: Runtime>(app: &AppHandle<R>, state: &MenuState) -> tauri::Result<Menu<R>> {
    let mac = cfg!(target_os = "macos");
    let about = AboutMetadata {
        name: Some("SpecDeck".into()),
        version: Some(app.package_info().version.to_string()),
        comments: Some("Run a coding agent with your team watching along.".into()),
        ..Default::default()
    };
    let settings = item(app, ids::SETTINGS, "Settings…", Some("CmdOrCtrl+,"), true)?;

    let mut recent = SubmenuBuilder::new(app, "Open Recent");
    for (i, folder) in state.recent.iter().enumerate() {
        let name = folder.file_name().map_or_else(
            || folder.to_string_lossy().into_owned(),
            |name| name.to_string_lossy().into_owned(),
        );
        recent =
            recent.text(format!("{RECENT_PREFIX}{i}"), format!("{name} — {}", folder.display()));
    }
    let recent = recent.enabled(!state.recent.is_empty()).build()?;

    let mut file = SubmenuBuilder::new(app, "File")
        .item(&item(app, ids::HOST, "Host a Session…", keys("Cmd+N", Some("Ctrl+Shift+N")), true)?)
        .item(&recent)
        .item(&item(app, ids::JOIN, "Join a Session…", keys("Cmd+J", Some("Ctrl+Shift+J")), true)?)
        .separator()
        .item(&item(
            app,
            ids::COPY_CODE,
            "Copy Invite Code",
            keys("Cmd+Shift+C", None),
            state.hosting,
        )?)
        .separator();
    file = if mac {
        file.close_window()
    } else {
        file.item(&settings).separator().close_window().item(&item(
            app,
            ids::QUIT,
            "Quit",
            None,
            true,
        )?)
    };
    let file = file.build()?;

    let mut edit = SubmenuBuilder::new(app, "Edit");
    edit = if mac {
        edit.undo().redo().separator().cut().copy().paste().select_all()
    } else {
        edit.item(&item(app, ids::COPY, "Copy", None, true)?)
            .item(&item(app, ids::PASTE, "Paste", None, true)?)
            .item(&item(app, ids::SELECT_ALL, "Select All", None, true)?)
    };
    let edit = edit
        .separator()
        .item(&item(app, ids::FIND, "Find…", keys("Cmd+F", None), true)?)
        .build()?;

    let mut view = SubmenuBuilder::new(app, "View")
        .item(&item(app, ids::TIMELINE, "Timeline", Some("CmdOrCtrl+1"), true)?)
        .item(&item(app, ids::FILES, "Files", Some("CmdOrCtrl+2"), true)?)
        .item(&item(app, ids::PEOPLE, "People", Some("CmdOrCtrl+3"), true)?)
        .separator()
        .item(&item(app, ids::JUMP_TO_LIVE, "Jump to Live", keys("Cmd+Down", None), true)?);
    if mac {
        view = view.separator().fullscreen();
    }
    let view = view.build()?;

    let mut help = SubmenuBuilder::new(app, "Help").item(&item(
        app,
        ids::SHORTCUTS,
        "Keyboard Shortcuts",
        keys("Cmd+/", Some("F1")),
        true,
    )?);
    if !mac {
        help = help.separator().about(Some(about.clone()));
    }
    let help = help.build()?;

    let mut menu = MenuBuilder::new(app);
    if mac {
        let app_menu = SubmenuBuilder::new(app, "SpecDeck")
            .about(Some(about))
            .separator()
            .item(&settings)
            .separator()
            .services()
            .separator()
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .item(&item(app, ids::QUIT, "Quit SpecDeck", Some("Cmd+Q"), true)?)
            .build()?;
        let window = SubmenuBuilder::new(app, "Window")
            .minimize()
            .maximize()
            .separator()
            .bring_all_to_front()
            .build()?;
        menu = menu.items(&[&app_menu, &file, &edit, &view, &window, &help]);
    } else {
        menu = menu.items(&[&file, &edit, &view, &help]);
    }
    menu.build()
}
