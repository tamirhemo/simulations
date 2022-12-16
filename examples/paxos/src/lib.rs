//! This crate contains an implementation of the Paxos algorithm using the interface provided in system.
//!

//#![warn(missing_docs)]

pub mod actors;
mod system;

pub use crate::system::setup_paxos;
