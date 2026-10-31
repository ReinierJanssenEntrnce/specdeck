//! Everything agent-specific: where an agent's executable is and what it needs before it can
//! start (design decision 3a). The pty, output log and streaming never know which agent runs.

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

/// Name of the agent's executable, looked up on `PATH`.
pub fn executable_name(agent: AgentId) -> &'static str {
    match agent {
        AgentId::ClaudeCode => "claude",
        AgentId::OpenCode => "opencode",
        AgentId::Omp => "omp",
    }
}

/// Which supported agents are installed, in display order.
pub fn detect(env: &Environment) -> Vec<Availability> {
    AgentId::ALL
        .into_iter()
        .map(|agent| Availability {
            agent,
            executable: env.find_executable(executable_name(agent)),
        })
        .collect()
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
        assert_eq!(found.len(), 3);
        assert_eq!(found[0].executable, Some(dir.path().join("claude")));
        assert_eq!(found[1].executable, Some(dir.path().join("opencode")));
        assert_eq!(found[2], Availability { agent: AgentId::Omp, executable: None });
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
