//! Takes over the settings of releases before the app identifier was renamed
//! (`rename-app-identifier` design decision 2). Their config folder is named after the old
//! identifier, next to the current one; it is copied once, before anything reads settings or
//! window state, and left in place so an older release still finds it.

use std::io;
use std::path::Path;

use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

/// The identifier of releases up to 1.3.
const OLD_IDENTIFIER: &str = "com.entrnce.specdeck";

/// Runs the copy in its setup hook. Register it before `tauri-plugin-window-state`, which reads
/// its file in its own setup hook: plugins are set up in the order they are registered.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("migrate-config")
        .setup(|app, _api| {
            if let Ok(dir) = app.path().app_config_dir()
                && let Err(error) = take_over_old_config(&dir)
            {
                // The app starts with defaults; the next start tries again.
                eprintln!("Could not take over the settings of {OLD_IDENTIFIER}: {error}");
            }
            Ok(())
        })
        .build()
}

/// Copies the old identifier's folder next to `dir` to `dir`, unless `dir` exists or there is
/// no old folder. Returns whether it copied. The copy goes to a temporary folder first, so a
/// failure leaves no `dir` behind and the next start tries again.
pub fn take_over_old_config(dir: &Path) -> io::Result<bool> {
    let (Some(parent), Some(name)) = (dir.parent(), dir.file_name()) else { return Ok(false) };
    let old = parent.join(OLD_IDENTIFIER);
    if dir.exists() || old == dir || !old.is_dir() {
        return Ok(false);
    }
    let mut temporary = name.to_owned();
    temporary.push(".migrating");
    let temporary = parent.join(temporary);
    // Left over from a copy that was interrupted.
    if temporary.exists() {
        std::fs::remove_dir_all(&temporary)?;
    }
    if let Err(error) = copy_dir(&old, &temporary).and_then(|()| std::fs::rename(&temporary, dir)) {
        let _ = std::fs::remove_dir_all(&temporary);
        return Err(error);
    }
    Ok(true)
}

/// Copies folders and files; anything else, such as a symbolic link, is skipped.
fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::create_dir(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else if kind.is_file() {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEW: &str = "nl.nickhartjes.specdeck";

    fn old_folder(root: &Path) -> std::path::PathBuf {
        let old = root.join(OLD_IDENTIFIER);
        std::fs::create_dir_all(old.join("profiles").join("viewer")).unwrap();
        std::fs::write(old.join("settings.json"), r#"{"username":"nick"}"#).unwrap();
        std::fs::write(old.join(".window-state.json"), "{}").unwrap();
        std::fs::write(old.join("profiles").join("viewer").join("settings.json"), "{}").unwrap();
        old
    }

    #[test]
    fn copies_the_old_folder_with_its_subfolders_and_keeps_it() {
        let root = tempfile::tempdir().unwrap();
        let old = old_folder(root.path());
        let new = root.path().join(NEW);

        assert!(take_over_old_config(&new).unwrap());
        assert_eq!(
            std::fs::read_to_string(new.join("settings.json")).unwrap(),
            r#"{"username":"nick"}"#
        );
        assert!(new.join(".window-state.json").is_file());
        assert!(new.join("profiles").join("viewer").join("settings.json").is_file());
        assert!(old.join("settings.json").is_file());
        assert!(!root.path().join(format!("{NEW}.migrating")).exists());
    }

    #[test]
    fn leaves_an_existing_folder_alone() {
        let root = tempfile::tempdir().unwrap();
        old_folder(root.path());
        let new = root.path().join(NEW);
        std::fs::create_dir(&new).unwrap();
        std::fs::write(new.join("settings.json"), r#"{"username":"new"}"#).unwrap();

        assert!(!take_over_old_config(&new).unwrap());
        assert_eq!(
            std::fs::read_to_string(new.join("settings.json")).unwrap(),
            r#"{"username":"new"}"#
        );
    }

    #[test]
    fn does_nothing_without_an_old_folder() {
        let root = tempfile::tempdir().unwrap();
        let new = root.path().join(NEW);

        assert!(!take_over_old_config(&new).unwrap());
        assert!(!new.exists());
    }

    #[test]
    fn replaces_an_interrupted_copy() {
        let root = tempfile::tempdir().unwrap();
        old_folder(root.path());
        let half = root.path().join(format!("{NEW}.migrating"));
        std::fs::create_dir(&half).unwrap();
        std::fs::write(half.join("partial"), "").unwrap();
        let new = root.path().join(NEW);

        assert!(take_over_old_config(&new).unwrap());
        assert!(new.join("settings.json").is_file());
        assert!(!new.join("partial").exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_copy_leaves_no_new_folder() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let old = old_folder(root.path());
        let unreadable = old.join("settings.json");
        std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000)).unwrap();
        // Running as root reads the file anyway; there is no failure to test then.
        if std::fs::read(&unreadable).is_ok() {
            return;
        }
        let new = root.path().join(NEW);

        assert!(take_over_old_config(&new).is_err());
        assert!(!new.exists());
        assert!(!root.path().join(format!("{NEW}.migrating")).exists());
        std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o644)).unwrap();
    }
}
