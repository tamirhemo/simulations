//! Implementation of actors as tokio threads sending messages to each other. 
//! 
//! The [`TokioSystem`] 
//! 
//! The user can signify the type of actor using the [`ActorType`]. 
//! 
//! 
//! To start a system
//! ```
//! let system = TokioSystem::new()
//! ```
//! 
//! //TODO: docs here
//! 


mod actor;
mod actor_core;
mod channel;
mod system;

pub use self::system::TokioSystem;
pub use actor_core::ActorType;
