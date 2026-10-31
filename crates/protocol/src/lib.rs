//! SpecDeck wire protocol shared by hosts and viewers. Contains no I/O.

mod agent;
pub mod timeline;

pub use agent::{AgentId, AgentStatus};
