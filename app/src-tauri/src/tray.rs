//! The menu-bar (macOS) or tray (Windows, Linux) icon while a session is hosted (design decision
//! 11). It shows the session's state and viewers, and offers the window, the invite code and
//! ending the session, also while the window is closed.

use specdeck_protocol::invite::InviteCode;
use specdeck_protocol::{AgentId, AgentStatus};
use tauri::image::Image;
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Runtime};

use crate::host::{SharingState, Summary};

const TRAY_ID: &str = "session";

pub mod ids {
    pub const OPEN: &str = "tray:open";
    pub const COPY_CODE: &str = "tray:copy-code";
    pub const END: &str = "tray:end";
}

/// The first line of the menu: what the agent is doing.
fn state_line(agent: AgentId, summary: &Summary) -> String {
    let agent_name = agent.display_name();
    match summary.status {
        AgentStatus::Exited if agent == AgentId::Terminal => "The shell has exited".into(),
        AgentStatus::Starting => format!("{agent_name} is starting"),
        AgentStatus::Working => format!("{agent_name} is working"),
        AgentStatus::Waiting => format!("{agent_name} is waiting for you"),
        AgentStatus::Exited => format!("{agent_name} has exited"),
        AgentStatus::Unknown => "Session is live".into(),
    }
}

fn viewers_line(summary: &Summary) -> String {
    let viewers = match summary.viewers {
        0 => "No viewers yet".to_string(),
        1 => "1 viewer".to_string(),
        n => format!("{n} viewers"),
    };
    match summary.sharing {
        SharingState::Unavailable => format!("{viewers} · sharing unavailable"),
        _ => viewers,
    }
}

fn menu<R: Runtime>(
    app: &AppHandle<R>,
    folder: &str,
    agent: AgentId,
    summary: &Summary,
) -> tauri::Result<Menu<R>> {
    let info =
        |id: &str, text: String| MenuItemBuilder::with_id(id, text).enabled(false).build(app);
    let code = summary.code.as_deref().and_then(|code| InviteCode::parse(code).ok());
    let copy_text = match &code {
        Some(code) => format!("Copy Invite Code  {code}"),
        None => "Copy Invite Code".into(),
    };
    MenuBuilder::new(app)
        .item(&info("tray:folder", format!("Hosting {folder}"))?)
        .item(&info("tray:state", state_line(agent, summary))?)
        .item(&info("tray:viewers", viewers_line(summary))?)
        .separator()
        .item(&MenuItemBuilder::with_id(ids::OPEN, "Open Window").build(app)?)
        .item(
            &MenuItemBuilder::with_id(ids::COPY_CODE, copy_text)
                .enabled(code.is_some())
                .build(app)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id(ids::END, "End Session…").build(app)?)
        .build()
}

fn tooltip(folder: &str, summary: &Summary) -> String {
    format!("SpecDeck: hosting {folder} · {}", viewers_line(summary))
}

/// Shows the icon for a session that just started.
pub fn show<R: Runtime>(
    app: &AppHandle<R>,
    folder: &str,
    agent: AgentId,
    summary: &Summary,
) -> tauri::Result<()> {
    remove(app);
    let icon = if cfg!(target_os = "macos") {
        Image::from_bytes(include_bytes!("../icons/tray-template.png"))?
    } else {
        app.default_window_icon()
            .cloned()
            .unwrap_or(Image::from_bytes(include_bytes!("../icons/32x32.png"))?)
    };
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(true)
        .tooltip(tooltip(folder, summary))
        .menu(&menu(app, folder, agent, summary)?)
        .show_menu_on_left_click(true)
        .build(app)?;
    Ok(())
}

/// Brings the icon's menu up to date.
pub fn update<R: Runtime>(app: &AppHandle<R>, folder: &str, agent: AgentId, summary: &Summary) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    if let Ok(menu) = menu(app, folder, agent, summary) {
        let _ = tray.set_menu(Some(menu));
    }
    let _ = tray.set_tooltip(Some(tooltip(folder, summary)));
}

/// Removes the icon: no session is hosted.
pub fn remove<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.remove_tray_by_id(TRAY_ID);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(status: AgentStatus, viewers: usize, sharing: SharingState) -> Summary {
        Summary { status, viewers, code: None, sharing }
    }

    #[test]
    fn says_what_the_agent_does_and_who_watches() {
        let waiting = summary(AgentStatus::Waiting, 3, SharingState::Ready);
        assert_eq!(state_line(AgentId::ClaudeCode, &waiting), "Claude Code is waiting for you");
        assert_eq!(viewers_line(&waiting), "3 viewers");
        assert_eq!(
            viewers_line(&summary(AgentStatus::Unknown, 1, SharingState::Ready)),
            "1 viewer"
        );
        assert_eq!(
            viewers_line(&summary(AgentStatus::Unknown, 0, SharingState::Unavailable)),
            "No viewers yet · sharing unavailable"
        );
        assert_eq!(
            state_line(AgentId::OpenCode, &summary(AgentStatus::Unknown, 0, SharingState::Ready)),
            "Session is live"
        );
    }

    #[test]
    fn a_terminal_session_talks_about_its_shell() {
        let exited = summary(AgentStatus::Exited, 0, SharingState::Ready);
        assert_eq!(state_line(AgentId::Terminal, &exited), "The shell has exited");
        assert_eq!(state_line(AgentId::Omp, &exited), "omp has exited");
        let live = summary(AgentStatus::Unknown, 2, SharingState::Ready);
        assert_eq!(state_line(AgentId::Terminal, &live), "Session is live");
    }
}
