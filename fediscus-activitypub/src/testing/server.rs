//! Testing fediverse server.
//! Runs a local HTTP server that can be used for testing the fediscus implementation.
//! This is more or less an exact copy of the "local federation" example from the `activitypub-federation` crate.

mod activities;
mod error;
mod http;
mod instance;
mod objects;
mod utils;

pub use instance::{listen, new_instance, DatabaseHandle};
