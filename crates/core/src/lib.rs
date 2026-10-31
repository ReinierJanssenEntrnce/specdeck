//! SpecDeck session engine: runs a coding agent for a host and connects hosts and viewers.

pub mod activity;
pub mod agent;
pub mod environment;
pub mod hooks;
pub mod ice;
pub mod launcher;
pub mod mirror;
pub mod output_log;
pub mod peer;
pub mod session;
pub mod sharing;
pub mod signalling_client;
pub mod status;
pub mod timeline;
pub mod usage;
pub mod viewer;

#[cfg(test)]
mod test_support;
