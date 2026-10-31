//! What the app remembers between runs: the user's name, recent folders, last used agent and
//! whether to show notifications.
//! Stored as JSON in the app's config directory.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specdeck_protocol::AgentId;

/// Recent folders listed on the start screen.
pub const MAX_RECENT: usize = 5;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub username: Option<String>,
    /// Newest first, each once.
    pub recent_folders: Vec<PathBuf>,
    pub last_agent: Option<AgentId>,
    pub notifications: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { username: None, recent_folders: Vec::new(), last_agent: None, notifications: true }
    }
}

impl Settings {
    /// Reads the settings, or starts fresh if there are none or they cannot be read.
    pub fn load(path: &Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        // Write then rename, so a crash never leaves half a file.
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(temporary, path)
    }

    /// Puts `folder` at the top of the recent folders.
    pub fn add_recent(&mut self, folder: PathBuf) {
        self.recent_folders.retain(|existing| existing != &folder);
        self.recent_folders.insert(0, folder);
        self.recent_folders.truncate(MAX_RECENT);
    }

    pub fn remove_recent(&mut self, folder: &Path) {
        self.recent_folders.retain(|existing| existing != folder);
    }

    /// Recent folders that still exist.
    pub fn existing_recent(&self) -> Vec<PathBuf> {
        self.recent_folders.iter().filter(|folder| folder.is_dir()).cloned().collect()
    }

    /// Sets the username, trimmed; an empty name clears it.
    pub fn set_username(&mut self, username: &str) {
        let username = username.trim();
        self.username = (!username.is_empty()).then(|| username.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_folders_newest_first_each_once_at_most_five() {
        let mut settings = Settings::default();
        for name in ["a", "b", "a", "c", "d", "e", "f"] {
            settings.add_recent(PathBuf::from(name));
        }
        let names: Vec<_> = settings.recent_folders.iter().map(|p| p.to_str().unwrap()).collect();
        assert_eq!(names, ["f", "e", "d", "c", "a"]);
    }

    #[test]
    fn removed_and_missing_folders_are_not_listed() {
        let dir = tempfile::tempdir().unwrap();
        let mut settings = Settings::default();
        settings.add_recent(dir.path().join("gone"));
        settings.add_recent(dir.path().to_owned());
        assert_eq!(settings.existing_recent(), [dir.path().to_owned()]);
        settings.remove_recent(dir.path());
        assert!(settings.existing_recent().is_empty());
        // Removing from the list does not touch the folder.
        assert!(dir.path().exists());
    }

    #[test]
    fn saves_and_loads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/settings.json");
        let mut settings = Settings::default();
        settings.set_username("  nick ");
        settings.last_agent = Some(AgentId::OpenCode);
        settings.add_recent(PathBuf::from("/projects/shop"));
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), settings);
        assert_eq!(settings.username.as_deref(), Some("nick"));
    }

    #[test]
    fn unreadable_settings_start_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        assert_eq!(Settings::load(&path), Settings::default());
        std::fs::write(&path, b"not json").unwrap();
        assert_eq!(Settings::load(&path), Settings::default());
    }

    #[test]
    fn notifications_are_on_unless_turned_off() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        // Settings from before the option existed.
        std::fs::write(&path, br#"{"username":"nick"}"#).unwrap();
        assert!(Settings::load(&path).notifications);
        std::fs::write(&path, br#"{"notifications":false}"#).unwrap();
        assert!(!Settings::load(&path).notifications);
    }

    #[test]
    fn empty_username_clears_it() {
        let mut settings = Settings::default();
        settings.set_username("nick");
        settings.set_username("   ");
        assert_eq!(settings.username, None);
    }
}
