//! Hooks for the commandline interface
//!
//! # Note to self
//! Put as little logic in here as possible.
//! That makes it easier to derive a pure library version later.

/// Contains concrete implementation of each subcommand
pub mod app;
pub mod subcommands;

#[cfg(feature = "shell")]
pub mod shell;

pub use self::app::{match_matches, with_cli};
