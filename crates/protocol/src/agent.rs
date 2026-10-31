use serde::{Deserialize, Serialize};

/// A coding agent SpecDeck can run in a hosted session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentId {
    ClaudeCode,
    OpenCode,
    Omp,
}

impl AgentId {
    pub const ALL: [AgentId; 3] = [AgentId::ClaudeCode, AgentId::OpenCode, AgentId::Omp];

    pub fn display_name(self) -> &'static str {
        match self {
            AgentId::ClaudeCode => "Claude Code",
            AgentId::OpenCode => "OpenCode",
            AgentId::Omp => "omp",
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
