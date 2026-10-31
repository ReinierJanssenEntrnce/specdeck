//! SpecDeck wire protocol shared by hosts and viewers. Contains no I/O.

mod agent;
pub mod control;
pub mod frames;
pub mod invite;
pub mod signal_auth;
pub mod signalling;
pub mod timeline;

pub use agent::{AgentId, AgentStatus};
