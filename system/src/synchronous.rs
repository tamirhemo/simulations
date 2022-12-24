//! An interface for synchronuous code message passing systems.
//!
//! One concrete implementation is provided by the crossbeam_channel crate.
//!

pub mod actor;
pub mod channel;
pub mod crossbeam;
mod standard;
pub mod system;

pub use actor::Actor;
pub use channel::{InChannel, OutChannels};
