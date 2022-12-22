//! An interace to generate simulations of distributed systems
//!realized as a set of actors and a set of message-passing channels.
//! The user only needs to write an implementation of each actors's inner logic
//! and a function that sets up the initial conditions.
//!
//! The internal logic of an actors is expressed by implementing the [`ActorInternal`] trait.
//! Systems are then built by instantiating a corresponding system.
//!
//! Currently there are two types of systems avaialble:
//! * [crossbeam::System](synchronous::crossbeam::System) - implements actors as threads with message
//! passing between them.
//!     * Simple to run and test.
//!     * Has limitations of scale as each actor runs on a dedicated thread.
//!
//! * [tokio::sync::System] - implementing actors using [tokio](https://tokio.rs) tasks and message passing.
//!     * Easily run many actors in a single simulation.
//!     * Users can specify different types of actors. For actors with internal operations that
//!     are potentially computationally heavy, blocking threads are spawn.
//!
//!
//! # Example
//! We demonstrate the use of the library by implementing a system consisting of three
//! actors passing a single message containing a `usize` integer in a cycle. Each actor will read the
//! value `m` and will transmit `m+1` to its outgoing channel. The execusion will end when the initial
//! actor gets a message back, and they will then return the value, which corresponds to the length
//! of the cycle.
//!
//! One the actors (designated starter) will send the first message and wait to recive it back.
//!
//! First, we define an actor's internal structure:
//!
//!```
//! # #[derive(Debug)]
//! pub struct CycleInternal {
//!    // ID of input actor
//!    input_key: Option<usize>,
//!    // ID of output actor
//!    output_key: Option<usize>,
//!    // whether the actor is the starter or not.
//!    starter : bool,
//! }
//!```
//!
//! To realize CycleInternal as an actor, we need to implement the [`ActorInternal`] trait.
//!
//! ```
//! # use self::system::internal::*;
//! # use self::system::synchronous::crossbeam::CrossbeamSystem;
//! # #[derive(Debug)]
//! # pub struct CycleInternal {
//! #   // ID of input actor
//! #  input_key: Option<usize>,
//! #  // ID of output actor
//! #  output_key: Option<usize>,
//! #  // whether the actor is the starter or not.
//! #  starter : bool,
//! # }
//! impl ActorInternal for CycleInternal {
//!     type Message = usize;
//!     type Error = SendError<(usize, usize)>;
//!     type Key = usize;
//!
//!     fn new_incoming_key(&mut self, key: &Self::Key) {
//!         assert!(self.input_key.is_none());
//!         self.input_key = Some(*key);
//!     }
//!
//!     fn new_outgoing_key(&mut self, key: &Self::Key) {
//!         assert!(self.output_key.is_none());
//!         self.output_key = Some(*key);
//!     }
//!
//!    fn start<S : Sender<Key = Self::Key, Message = Self::Message>>
//!    (&mut self, tx: &mut S)
//!     -> Result<NextState<Self::Message>, Self::Error> {
//!        if self.starter {
//!            let out = self.output_key.unwrap();
//!            tx.send(&out, 0).unwrap();
//!        }
//!        Ok(NextState::Get)
//!    }
//!
//!    fn process_message<S : Sender<Key = Self::Key, Message = Self::Message>>
//!     (&mut self, message: Option<Self::Message>, tx: &mut  S)
//!      -> Result<NextState<Self::Message>, Self::Error> {
//!        assert!(message.is_some());
//!        let value = message.unwrap();
//!    
//!        let out = self.output_key.unwrap();
//!        // we don't want to panick just because the next actor might be done already
//!        tx.send(&out, value+1).ok();
//!        Ok(NextState::Terminate(Some(value+1)))
//!    }
//! }
//!```
//!
//!
//! We can now initiate and start the system as follows:
//!
//!```
//! use system::synchronous::crossbeam::CrossbeamSystem;
//! # use system::internal::*;
//! # use system::System;
//! # #[derive(Debug)]
//! # pub struct CycleInternal {
//! #   // ID of input actor
//! #  input_key: Option<usize>,
//! #  // ID of output actor
//! #  output_key: Option<usize>,
//! #  // whether the actor is the starter or not.
//! #  starter : bool,
//! # }
//! # impl CycleInternal {
//! #     fn new(starter : bool) -> Self {
//! #         CycleInternal { input_key: None, output_key: None, starter: starter }
//! #     }
//! #  }
//! # impl ActorInternal for CycleInternal {
//! #     type Message = usize;
//! #     type Error = SendError<(usize, usize)>;
//! #     type Key = usize;
//! #
//! #     fn new_incoming_key(&mut self, key: &Self::Key) {
//! #         assert!(self.input_key.is_none());
//! #        self.input_key = Some(*key);
//! #     }
//! #
//! #    fn new_outgoing_key(&mut self, key: &Self::Key) {
//! #         assert!(self.output_key.is_none());
//! #         self.output_key = Some(*key);
//! #     }
//! #
//! #    fn start<S : Sender<Key = Self::Key, Message = Self::Message>>
//! #     (&mut self, tx: &mut S) -> Result<NextState<Self::Message>, Self::Error> {
//! #        if self.starter {
//! #            let out = self.output_key.unwrap();
//! #            tx.send(&out, 0).ok();
//! #        }
//! #        Ok(NextState::Get)
//! #    }
//! #
//! #   fn process_message<S : Sender<Key = Self::Key, Message = Self::Message>>
//! #    (&mut self, message: Option<Self::Message>, tx: &mut  S) -> Result<NextState<Self::Message>, Self::Error> {
//! #        assert!(message.is_some());
//! #        let value = message.unwrap();
//! #    
//! #        let out = self.output_key.unwrap();
//! #        tx.send(&out, value+1).ok();
//! #        Ok(NextState::Terminate(Some(value+1)))
//! #    }
//! # }
//! #
//! let mut cycle = CrossbeamSystem::new();
//!
//! // Add actors
//! cycle.add_actor(0, CycleInternal::new(true), None);
//! cycle.add_actor(1, CycleInternal::new(false), None);
//! cycle.add_actor(2, CycleInternal::new(false), None);
//!
//! // Add channels
//! cycle.add_channel(&0, &1);
//! cycle.add_channel(&1, &2);
//! cycle.add_channel(&2, &0);
//!
//! // Make actors 0 terminal
//! cycle.add_terminal(0);
//!
//! let values = cycle.run().unwrap();
//! assert_eq!(values[&0], Some(3));
//!```
//!

pub mod internal;
pub mod synchronous;
pub mod tokio;

//pub use crate::tokio::sync::TokioSystem;
pub use synchronous::crossbeam::CrossbeamSystem;

pub use internal::{ActorInternal, NextState, Sender, SendError};

/// An interface defining methods of a system useful for set-up
/// 
///
/// **Note**: The reason we did not abstract away some other properties of actors and systems in a trait is
/// that including async methods in trait is an unstable fearture in Rust.
pub trait System: Sized {
    type Internal: ActorInternal;
    type ActorParameters;

    /// Add a new actors to the system, with a given internal core and identifying key
    fn add_actor(
        &mut self,
        key: <Self::Internal as ActorInternal>::Key,
        internal: Self::Internal,
        parameters: Option<Self::ActorParameters>,
    );

    /// Add a channel between the actor identified with the key [`sender`] and [`reciever`]
    fn add_channel(
        &mut self,
        sender: &<Self::Internal as ActorInternal>::Key,
        reciever: &<Self::Internal as ActorInternal>::Key,
    );

    /// Add an actor to the set of terminals
    ///
    /// The system does not finish execusion until each of the actors in the set of terminals
    /// has been terminated. Once all the actors in the set of terminals is done executing,
    ///  an actor not in this set will be dropped regardless of whether it terminated or not.
    fn add_terminal(&mut self, key: <Self::Internal as ActorInternal>::Key);
}
