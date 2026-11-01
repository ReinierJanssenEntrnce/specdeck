//! Tracks the agent's status and whether the timeline is working (degraded mode, design
//! decision 10).
//!
//! The timeline becomes unavailable when the host presses Enter and no hook event arrives within
//! [`EVENT_TIMEOUT`], or when a payload cannot be parsed. The next valid event makes it available
//! again. Agents without an event source never have a timeline and their status is unknown.

use std::time::{Duration, Instant};

use specdeck_protocol::timeline::Entry;
use specdeck_protocol::{AgentId, AgentStatus};

/// How long after the host presses Enter a hook event must arrive.
pub const EVENT_TIMEOUT: Duration = Duration::from_secs(10);

/// What changed, for the engine to send to participants.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    Status(AgentStatus),
    /// A timeline entry about the timeline itself: unavailable or available again.
    Timeline(Entry),
}

pub struct Activity {
    has_timeline: bool,
    status: AgentStatus,
    timeline_available: bool,
    /// When the host pressed Enter with no hook event since.
    awaiting_event_since: Option<Instant>,
}

impl Activity {
    pub fn new(agent: AgentId) -> Self {
        let has_timeline = agent.has_timeline();
        Self {
            has_timeline,
            status: if has_timeline { AgentStatus::Starting } else { AgentStatus::Unknown },
            timeline_available: has_timeline,
            awaiting_event_since: None,
        }
    }

    pub fn status(&self) -> AgentStatus {
        self.status
    }

    pub fn timeline_available(&self) -> bool {
        self.timeline_available
    }

    /// The agent wrote to the terminal.
    pub fn on_output(&mut self) -> Vec<Change> {
        if self.status == AgentStatus::Starting {
            self.set_status(AgentStatus::Waiting)
        } else {
            Vec::new()
        }
    }

    /// The host pressed Enter in the terminal.
    pub fn on_host_enter(&mut self, now: Instant) {
        if self.has_timeline && self.awaiting_event_since.is_none() {
            self.awaiting_event_since = Some(now);
        }
    }

    /// A hook payload was turned into these entries.
    pub fn on_entries(&mut self, entries: &[Entry]) -> Vec<Change> {
        self.awaiting_event_since = None;
        let mut changes = Vec::new();
        if !self.timeline_available {
            self.timeline_available = true;
            changes.push(Change::Timeline(Entry::TimelineAvailable));
        }
        for entry in entries {
            let status = match entry {
                Entry::Prompt { .. }
                | Entry::Message { .. }
                | Entry::ToolCall { .. }
                | Entry::ToolResult { .. } => AgentStatus::Working,
                Entry::Response { .. } | Entry::Attention { .. } => AgentStatus::Waiting,
                Entry::SessionEnded => AgentStatus::Exited,
                _ => continue,
            };
            changes.extend(self.set_status(status));
        }
        changes
    }

    /// A hook payload could not be parsed.
    pub fn on_bad_payload(&mut self, reason: String) -> Vec<Change> {
        self.awaiting_event_since = None;
        self.mark_unavailable(reason)
    }

    /// Checks the event timeout; call at least once a second.
    pub fn poll(&mut self, now: Instant) -> Vec<Change> {
        match self.awaiting_event_since {
            Some(since) if now.duration_since(since) >= EVENT_TIMEOUT => {
                self.awaiting_event_since = None;
                self.mark_unavailable("No events arrived from the agent.".into())
            }
            _ => Vec::new(),
        }
    }

    /// The agent process exited.
    pub fn on_exit(&mut self) -> Vec<Change> {
        self.awaiting_event_since = None;
        self.set_status(AgentStatus::Exited)
    }

    fn mark_unavailable(&mut self, reason: String) -> Vec<Change> {
        if !self.timeline_available {
            return Vec::new();
        }
        self.timeline_available = false;
        vec![Change::Timeline(Entry::TimelineUnavailable { reason })]
    }

    fn set_status(&mut self, status: AgentStatus) -> Vec<Change> {
        // Without events only the exit is known.
        let status = if !self.has_timeline && status != AgentStatus::Exited {
            AgentStatus::Unknown
        } else {
            status
        };
        if status == self.status {
            return Vec::new();
        }
        self.status = status;
        vec![Change::Status(status)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prompt() -> Entry {
        Entry::Prompt { prompt_id: "p".into(), text: "hi".into() }
    }

    fn response() -> Entry {
        Entry::Response { prompt_id: "p".into(), text: "done".into() }
    }

    fn unavailable(changes: &[Change]) -> bool {
        matches!(changes, [Change::Timeline(Entry::TimelineUnavailable { .. })])
    }

    #[test]
    fn status_follows_the_turn() {
        let mut activity = Activity::new(AgentId::ClaudeCode);
        assert_eq!(activity.status(), AgentStatus::Starting);
        assert_eq!(activity.on_output(), [Change::Status(AgentStatus::Waiting)]);
        assert!(activity.on_output().is_empty());
        assert_eq!(activity.on_entries(&[prompt()]), [Change::Status(AgentStatus::Working)]);
        assert_eq!(activity.on_entries(&[response()]), [Change::Status(AgentStatus::Waiting)]);
        assert_eq!(activity.on_exit(), [Change::Status(AgentStatus::Exited)]);
    }

    #[test]
    fn a_permission_request_waits_until_the_tool_ran() {
        let mut activity = Activity::new(AgentId::ClaudeCode);
        activity.on_entries(&[prompt()]);
        let asked = Entry::Attention { message: "Claude needs your permission to use Bash".into() };
        assert_eq!(activity.on_entries(&[asked]), [Change::Status(AgentStatus::Waiting)]);
        let ran = Entry::ToolResult {
            prompt_id: "p".into(),
            tool_use_id: "t".into(),
            status: specdeck_protocol::timeline::ToolStatus::Succeeded,
            error: None,
            outcome: None,
            permission: None,
        };
        assert_eq!(activity.on_entries(&[ran]), [Change::Status(AgentStatus::Working)]);
    }

    #[test]
    fn no_event_within_ten_seconds_of_enter_makes_the_timeline_unavailable() {
        let start = Instant::now();
        let mut activity = Activity::new(AgentId::ClaudeCode);
        activity.on_host_enter(start);
        assert!(activity.poll(start + Duration::from_secs(9)).is_empty());
        assert!(unavailable(&activity.poll(start + EVENT_TIMEOUT)));
        assert!(!activity.timeline_available());
        // Reported once, not on every poll.
        activity.on_host_enter(start + EVENT_TIMEOUT);
        assert!(activity.poll(start + EVENT_TIMEOUT * 3).is_empty());
    }

    #[test]
    fn an_event_in_time_keeps_the_timeline_available() {
        let start = Instant::now();
        let mut activity = Activity::new(AgentId::ClaudeCode);
        activity.on_host_enter(start);
        activity.on_entries(&[prompt()]);
        assert!(activity.poll(start + EVENT_TIMEOUT * 2).is_empty());
        assert!(activity.timeline_available());
    }

    #[test]
    fn a_bad_payload_makes_the_timeline_unavailable_and_the_next_event_restores_it() {
        let mut activity = Activity::new(AgentId::ClaudeCode);
        assert!(unavailable(&activity.on_bad_payload("Unexpected Stop payload".into())));
        let changes = activity.on_entries(&[prompt()]);
        assert_eq!(
            changes,
            [Change::Timeline(Entry::TimelineAvailable), Change::Status(AgentStatus::Working)]
        );
        assert!(activity.timeline_available());
    }

    #[test]
    fn agents_without_events_have_no_timeline_and_unknown_status() {
        let start = Instant::now();
        let mut activity = Activity::new(AgentId::OpenCode);
        assert_eq!(activity.status(), AgentStatus::Unknown);
        assert!(!activity.timeline_available());
        assert!(activity.on_output().is_empty());
        activity.on_host_enter(start);
        assert!(activity.poll(start + EVENT_TIMEOUT * 2).is_empty());
        assert_eq!(activity.on_exit(), [Change::Status(AgentStatus::Exited)]);
    }
}
