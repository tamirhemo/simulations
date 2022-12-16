//! An interface for synchronuous code message passing systems.
//!
//!
//! The interface is based on a multiple-producer single consumer model for the channels.
//!
//! Implementations via std::sync::mpsc and crossbeam_channel are given.
//!

pub mod actor;
pub mod channel;
pub mod crossbeam;
mod standard;
pub mod system;

pub use actor::Actor;
pub use channel::{InChannel, OutChannels};
pub use crossbeam::ActorCB;
