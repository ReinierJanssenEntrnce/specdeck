//! SpecDeck session engine: runs a coding agent for a host and connects hosts and viewers.

pub mod activity;
pub mod agent;
pub mod environment;
pub mod hooks;
pub mod launcher;
pub mod mirror;
pub mod output_log;
pub mod session;
pub mod timeline;

#[cfg(test)]
mod test_support;
