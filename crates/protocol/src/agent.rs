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
    // Added in protocol version 2, after Terminal so the variants above keep their encoding.
    Codex,
    Grok,
    Pi,
}

impl AgentId {
    /// In display order: the coding agents, then Terminal.
    pub const ALL: [AgentId; 7] = [
        AgentId::ClaudeCode,
        AgentId::OpenCode,
        AgentId::Omp,
        AgentId::Codex,
        AgentId::Grok,
        AgentId::Pi,
        AgentId::Terminal,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            AgentId::ClaudeCode => "Claude Code",
            AgentId::OpenCode => "OpenCode",
            AgentId::Omp => "omp",
            AgentId::Terminal => "Terminal",
            AgentId::Codex => "Codex",
            AgentId::Grok => "Grok",
            AgentId::Pi => "pi",
        }
    }

    /// Whether SpecDeck captures this agent's events, and so can build a timeline and status.
    pub fn has_timeline(self) -> bool {
        matches!(self, AgentId::ClaudeCode)
    }

    /// Whether the agent reports whether it is working or waiting, through its events or the
    /// Program Status Protocol (`agent-status-signals`).
    pub fn reports_status(self) -> bool {
        // Only agents whose recordings show reports (crates/core/tests/fixtures/program-status).
        matches!(
            self,
            AgentId::ClaudeCode | AgentId::Omp | AgentId::Codex | AgentId::Grok | AgentId::Pi
        )
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
        assert_eq!(
            names,
            [
                r#""claude-code""#,
                r#""open-code""#,
                r#""omp""#,
                r#""codex""#,
                r#""grok""#,
                r#""pi""#,
                r#""terminal""#
            ]
        );
    }

    #[test]
    fn only_claude_code_has_a_timeline() {
        let with_timeline: Vec<AgentId> =
            AgentId::ALL.into_iter().filter(|agent| agent.has_timeline()).collect();
        assert_eq!(with_timeline, [AgentId::ClaudeCode]);
        assert_eq!(AgentId::Terminal.display_name(), "Terminal");
        assert_eq!(AgentId::Pi.display_name(), "pi");
    }

    #[test]
    fn claude_code_and_omp_report_their_status() {
        let reporting: Vec<AgentId> =
            AgentId::ALL.into_iter().filter(|agent| agent.reports_status()).collect();
        assert_eq!(
            reporting,
            [AgentId::ClaudeCode, AgentId::Omp, AgentId::Codex, AgentId::Grok, AgentId::Pi]
        );
    }
}
