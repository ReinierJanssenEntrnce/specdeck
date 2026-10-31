//! Everything agent-specific: where an agent's executable is and what it needs before it can
//! start (design decision 3a). The pty, output log and streaming never know which agent runs.
//! A terminal session is one more "agent": the host's own shell.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use specdeck_protocol::AgentId;

use crate::environment::Environment;

/// Something the agent needs besides its own executable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prerequisite {
    /// Claude Code on Windows runs its tools through Git Bash.
    GitBash,
}

impl Prerequisite {
    pub fn display_name(self) -> &'static str {
        match self {
            Prerequisite::GitBash => "Git Bash (Git for Windows)",
        }
    }
}

/// Whether an agent can be started on this machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Availability {
    pub agent: AgentId,
    /// Full path of the executable, if it was found.
    pub executable: Option<PathBuf>,
}

/// What to run for a session: the executable and the arguments it needs before any others.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub executable: PathBuf,
    pub args: Vec<OsString>,
}

/// Where `agent` is on this machine and how to start it; `None` when it is not installed (or,
/// for a terminal, when no shell was found).
pub fn program(agent: AgentId, env: &Environment) -> Option<Program> {
    let name = match agent {
        AgentId::ClaudeCode => "claude",
        AgentId::OpenCode => "opencode",
        AgentId::Omp => "omp",
        AgentId::Terminal => return terminal(env),
    };
    env.find_executable(name).map(|executable| Program { executable, args: Vec::new() })
}

/// Which supported agents are installed, in display order.
pub fn detect(env: &Environment) -> Vec<Availability> {
    AgentId::ALL
        .into_iter()
        .map(|agent| Availability {
            agent,
            executable: program(agent, env).map(|program| program.executable),
        })
        .collect()
}

/// The shell the host's own terminal app would start (design decision 2 of
/// `host-terminal-session`): PowerShell on Windows, the login shell elsewhere.
fn terminal(env: &Environment) -> Option<Program> {
    if cfg!(windows) {
        let executable = windows_shell(env, Path::is_file)?;
        return Some(Program { executable, args: vec!["-NoLogo".into()] });
    }
    let executable =
        unix_shell(env.get("SHELL").map(PathBuf::from), cfg!(target_os = "macos"), is_executable)?;
    // A login shell, as macOS's Terminal and most Linux terminals start: the host's profile,
    // prompt and aliases.
    Some(Program { executable, args: vec!["-l".into()] })
}

/// `$SHELL` when it can be run, else the platform's usual shells in order.
fn unix_shell(
    shell: Option<PathBuf>,
    macos: bool,
    is_executable: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let fallbacks: &[&str] =
        if macos { &["/bin/zsh", "/bin/bash", "/bin/sh"] } else { &["/bin/bash", "/bin/sh"] };
    shell
        .filter(|shell| shell.is_absolute())
        .into_iter()
        .chain(fallbacks.iter().map(PathBuf::from))
        .find(|path| is_executable(path))
}

/// PowerShell 7 when installed, else Windows PowerShell, which comes with Windows.
fn windows_shell(env: &Environment, exists: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    env.find_executable("pwsh.exe").or_else(|| env.find_executable("powershell.exe")).or_else(
        || {
            let root = PathBuf::from(env.get("SystemRoot")?);
            Some(root.join(r"System32\WindowsPowerShell\v1.0\powershell.exe"))
                .filter(|path| exists(path))
        },
    )
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// The first prerequisite of `agent` that is missing on this machine, if any.
pub fn missing_prerequisite(agent: AgentId, env: &Environment) -> Option<Prerequisite> {
    match agent {
        AgentId::ClaudeCode if cfg!(windows) => {
            find_git_bash(env, |path| path.is_file()).is_none().then_some(Prerequisite::GitBash)
        }
        _ => None,
    }
}

/// Finds Git Bash the way Claude Code does: `CLAUDE_CODE_GIT_BASH_PATH`, the default install
/// locations, then next to `git` on `PATH`.
fn find_git_bash(env: &Environment, exists: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    if let Some(path) = env.get("CLAUDE_CODE_GIT_BASH_PATH").map(PathBuf::from) {
        return exists(&path).then_some(path);
    }
    let mut candidates = vec![
        PathBuf::from(r"C:\Program Files\Git\bin\bash.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Git\bin\bash.exe"),
    ];
    if let Some(local) = env.get("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local).join(r"Programs\Git\bin\bash.exe"));
    }
    // git.exe lives in `<Git>\cmd`; bash.exe in `<Git>\bin`.
    if let Some(git) = env.find_executable("git")
        && let Some(root) = git.parent().and_then(Path::parent)
    {
        candidates.push(root.join("bin").join("bash.exe"));
    }
    candidates.into_iter().find(|path| exists(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn detects_installed_agents() {
        let dir = tempfile::tempdir().unwrap();
        crate::test_support::fake_executable(dir.path(), "claude", "exit 0");
        crate::test_support::fake_executable(dir.path(), "opencode", "exit 0");
        let env: Environment = [("PATH", dir.path())].into_iter().collect();

        let found = detect(&env);
        assert_eq!(found.len(), 4);
        assert_eq!(found[0].executable, Some(dir.path().join("claude")));
        assert_eq!(found[1].executable, Some(dir.path().join("opencode")));
        assert_eq!(found[2], Availability { agent: AgentId::Omp, executable: None });
        // A shell always resolves: no $SHELL here, so a system one.
        assert_eq!(found[3].agent, AgentId::Terminal);
        assert!(found[3].executable.is_some());
    }

    #[cfg(unix)]
    #[test]
    fn terminal_runs_the_login_shell_from_shell() {
        let dir = tempfile::tempdir().unwrap();
        let shell = crate::test_support::fake_executable(dir.path(), "myshell", "exit 0");
        let env: Environment = [("SHELL", shell.as_os_str())].into_iter().collect();
        assert_eq!(
            program(AgentId::Terminal, &env),
            Some(Program { executable: shell, args: vec!["-l".into()] })
        );
    }

    #[test]
    fn unix_shell_falls_back_in_order() {
        let only = |paths: &'static [&'static str]| {
            move |path: &Path| paths.iter().any(|p| path == Path::new(p))
        };
        let missing = Some(PathBuf::from("/opt/gone/fish"));
        assert_eq!(
            unix_shell(missing.clone(), true, only(&["/bin/zsh", "/bin/sh"])),
            Some(PathBuf::from("/bin/zsh"))
        );
        assert_eq!(
            unix_shell(None, true, only(&["/bin/bash", "/bin/sh"])),
            Some(PathBuf::from("/bin/bash"))
        );
        // zsh is the macOS default only; Linux starts with bash.
        assert_eq!(
            unix_shell(missing, false, only(&["/bin/zsh", "/bin/sh"])),
            Some(PathBuf::from("/bin/sh"))
        );
        assert_eq!(unix_shell(None, false, only(&[])), None);
    }

    #[cfg(unix)]
    #[test]
    fn windows_prefers_powershell_7() {
        let dir = tempfile::tempdir().unwrap();
        let env: Environment = [("PATH", dir.path())].into_iter().collect();
        assert_eq!(windows_shell(&env, |_| false), None);

        let classic = crate::test_support::fake_executable(dir.path(), "powershell.exe", "exit 0");
        assert_eq!(windows_shell(&env, |_| false), Some(classic));
        let seven = crate::test_support::fake_executable(dir.path(), "pwsh.exe", "exit 0");
        assert_eq!(windows_shell(&env, |_| false), Some(seven));
    }

    #[test]
    fn windows_powershell_in_its_system_folder() {
        let env: Environment = [("SystemRoot", r"C:\Windows")].into_iter().collect();
        let expected =
            Path::new(r"C:\Windows").join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
        assert_eq!(windows_shell(&env, |path| path == expected), Some(expected.clone()));
    }

    #[test]
    fn git_bash_from_override() {
        let env: Environment =
            [("CLAUDE_CODE_GIT_BASH_PATH", r"D:\tools\bash.exe")].into_iter().collect();
        let found = find_git_bash(&env, |path| path == Path::new(r"D:\tools\bash.exe"));
        assert_eq!(found, Some(PathBuf::from(r"D:\tools\bash.exe")));
    }

    #[test]
    fn git_bash_override_that_does_not_exist_is_missing() {
        let env: Environment =
            [("CLAUDE_CODE_GIT_BASH_PATH", r"D:\nope\bash.exe")].into_iter().collect();
        assert_eq!(find_git_bash(&env, |_| false), None);
    }

    #[test]
    fn git_bash_in_default_location() {
        let env = Environment::default();
        let found =
            find_git_bash(&env, |path| path == Path::new(r"C:\Program Files\Git\bin\bash.exe"));
        assert_eq!(found, Some(PathBuf::from(r"C:\Program Files\Git\bin\bash.exe")));
    }

    #[test]
    fn git_bash_missing() {
        assert_eq!(find_git_bash(&Environment::default(), |_| false), None);
    }

    #[test]
    fn only_claude_code_needs_git_bash_and_only_on_windows() {
        let env = Environment::default();
        assert_eq!(missing_prerequisite(AgentId::OpenCode, &env), None);
        assert_eq!(missing_prerequisite(AgentId::Omp, &env), None);
        let expected = cfg!(windows).then_some(Prerequisite::GitBash);
        // A test machine may have Git installed; only assert when it clearly is not.
        if find_git_bash(&env, |path| path.is_file()).is_none() {
            assert_eq!(missing_prerequisite(AgentId::ClaudeCode, &env), expected);
        }
    }
}
