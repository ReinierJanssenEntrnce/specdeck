use serde::{Deserialize, Serialize};

/// What a hosted session runs: a coding agent, or a plain terminal with the host's shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentId {
    ClaudeCode,
    OpenCode,
    Omp,
    /// The host's own shell; no events, so no timeline.
    Terminal,
}

impl AgentId {
    pub const ALL: [AgentId; 4] =
        [AgentId::ClaudeCode, AgentId::OpenCode, AgentId::Omp, AgentId::Terminal];

    pub fn display_name(self) -> &'static str {
        match self {
            AgentId::ClaudeCode => "Claude Code",
            AgentId::OpenCode => "OpenCode",
            AgentId::Omp => "omp",
            AgentId::Terminal => "Terminal",
        }
    }

    /// Whether SpecDeck captures this agent's events, and so can build a timeline and status.
    pub fn has_timeline(self) -> bool {
        matches!(self, AgentId::ClaudeCode)
    }
}

/// What the agent is doing, as participants see it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentStatus {
    /// Started, nothing on screen yet.
    Starting,
    /// Processing a prompt.
    Working,
    /// Waiting for the host.
    Waiting,
    /// The agent process has exited.
    Exited,
    /// The agent's events are not captured, so its activity is not known.
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_names() {
        let names: Vec<String> =
            AgentId::ALL.iter().map(|agent| serde_json::to_string(agent).unwrap()).collect();
        assert_eq!(names, [r#""claude-code""#, r#""open-code""#, r#""omp""#, r#""terminal""#]);
    }

    #[test]
    fn only_claude_code_has_a_timeline() {
        let with_timeline: Vec<AgentId> =
            AgentId::ALL.into_iter().filter(|agent| agent.has_timeline()).collect();
        assert_eq!(with_timeline, [AgentId::ClaudeCode]);
        assert_eq!(AgentId::Terminal.display_name(), "Terminal");
    }
}
