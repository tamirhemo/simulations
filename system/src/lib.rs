//! An interace to generate simulations of distributed systems 
//!realized as a set of agents and a set of message-passing channels. 
//! The user only needs to write an implementation of each Agent's inner logic 
//! and a function that sets up the initial conditions.
//! 
//! The internal logic of an agent is expressed by implementing the [`Internal`] trait. 
//!
//! 
//! # Example
//! ffff
//! 
//!

pub mod internal;
pub mod synchronous;
pub mod tokio;


pub use internal::{Internal, Instruction};


