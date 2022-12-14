//! An interace to generate simulations of distributed systems
//!realized as a set of agents and a set of message-passing channels.
//! The user only needs to write an implementation of each Agent's inner logic
//! and a function that sets up the initial conditions.
//!
//! The internal logic of an agent is expressed by implementing the [`AgentInternal`] trait.
//! Systems are then built by instantiating a corresponding System struct.
//!
//! Currently there are two types of Systems avaialble:
//! * [crossbeam::System](synchronous::crossbeam::System) - implementing agents as threads with message
//! passing between them.
//!     * Simple to run and test.
//!     * Has limitations of scale as each agent runs on a dedicated thread.
//!
//! * [tokio::sync::System] - implementing agents using [tokio](https://tokio.rs) tasks and message passing.
//!     * Easily run many agents in a single simulation.
//!     * Users can specify different types of agents. For agents with internal operations that
//!     are potentially computationally heavy, blocking threads are spawn.
//!
//!
//! # Example
//! We demonstrate the use of the library by implementing a system consisting of three
//! agents passing a single message in a circle.
//!
//! One the agents (designated starter) will send the first message and wait to recive it back.
//!
//! First, we define an agent's internal structure:
//!
//!```
//! # #[derive(Debug)]
//! pub struct CycleInternal {
//!    // ID of input agent
//!    input_key: Option<usize>,
//!    // ID of output agent
//!    output_key: Option<usize>,
//!    // whether the agent is the starter or not.
//!    starter : bool,
//! }
//!```
//!
//! To realize CycleInternal as an agent, we need to implement the [`AgentInternal`] trait.
//!
//! ```
//! # use system::internal::*;
//! # #[derive(Debug)]
//! # pub struct CycleInternal {
//! #   // ID of input agent
//! #  input_key: Option<usize>,
//! #  // ID of output agent
//! #  output_key: Option<usize>,
//! #  // whether the agent is the starter or not.
//! #  starter : bool,
//! # }
//! impl AgentInternal for CycleInternal {
//!     type Message = usize;
//!     type Error = ();
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
//!    fn start(&mut self, tx: &mut Sender<Self::Key, Self::Message>) -> Result<NextState<Self::Message>, Self::Error> {
//!        if self.starter {
//!            let out = self.output_key.unwrap();
//!            tx.send(out, 0).unwrap();
//!        }
//!        Ok(NextState::Get)
//!    }
//!
//!    fn process_message(&mut self, message: Option<Self::Message>, tx: &mut  Sender<Self::Key, Self::Message>) -> Result<NextState<Self::Message>, Self::Error> {
//!        assert!(message.is_some());
//!        let value = message.unwrap();
//!    
//!        let out = self.output_key.unwrap();
//!        tx.send(out, value+1).unwrap();
//!        Ok(NextState::Terminate(value+1))
//!    }
//! }
//!```
//!
//!
//! We can now initiate and start the system as follows:
//!
//!```
//! # use system::synchronous::crossbeam::System;
//! # use system::internal::*;
//! # #[derive(Debug)]
//! # pub struct CycleInternal {
//! #   // ID of input agent
//! #  input_key: Option<usize>,
//! #  // ID of output agent
//! #  output_key: Option<usize>,
//! #  // whether the agent is the starter or not.
//! #  starter : bool,
//! # }
//! # impl CycleInternal {
//! #     fn new(starter : bool) -> Self {
//! #         CycleInternal { input_key: None, output_key: None, starter: starter }
//! #     }
//! #  }
//! # impl AgentInternal for CycleInternal {
//! #     type Message = usize;
//! #     type Error = ();
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
//! #    fn start(&mut self, tx: &mut Sender<Self::Key, Self::Message>) -> Result<NextState<Self::Message>, Self::Error> {
//! #        if self.starter {
//! #            let out = self.output_key.unwrap();
//! #            tx.send(out, 0).unwrap();
//! #        }
//! #        Ok(NextState::Get)
//! #    }
//! #
//! #   fn process_message(&mut self, message: Option<Self::Message>, tx: &mut  Sender<Self::Key, Self::Message>) -> Result<NextState<Self::Message>, Self::Error> {
//! #        assert!(message.is_some());
//! #        let value = message.unwrap();
//! #    
//! #        let out = self.output_key.unwrap();
//! #        tx.send(out, value+1).unwrap();
//! #        Ok(NextState::Terminate(value+1))
//! #    }
//! # }
//! #
//! let mut cycle = System::new();
//!
//! // Add agents
//! cycle.add_agent(0, CycleInternal::new(true));
//! cycle.add_agent(1, CycleInternal::new(false));
//! cycle.add_agent(2, CycleInternal::new(false));
//!
//! // Add channels
//! cycle.add_channel(&0, &1);
//! cycle.add_channel(&1, &2);
//! cycle.add_channel(&2, &0);
//!
//! // Make agents 0 terminal
//! cycle.add_terminal(0);
//!
//! let values = cycle.run().unwrap();
//! assert_eq!(values.get(&0), Some(&3));
//!```
//!

pub mod internal;
pub mod synchronous;
pub mod tokio;

pub use internal::{AgentInternal, Internal, Instruction, NextState, Sender};


/// An interface defining properties of a system useful for setup
/// 
/// **Note**: The reason we did not abstract away some other properties of agents and systems in a trait is
/// that including async methods in trait is an unstable fearture in Rust.
pub trait System : Sized {
    type Internal: Internal;
    type AgentParameters;

    /// Add a new agent to the system, with a given internal core and identifying key
    fn add_agent(
        &mut self,
        key: <Self::Internal as Internal>::Key,
        internal: Self::Internal,
        parameters: Self::AgentParameters,
    );

    /// Add a channel between the agent identified with the key [`sender`] and [`reciever`]
    fn add_channel(
        &mut self,
        sender: &<Self::Internal as Internal>::Key,
        reciever: &<Self::Internal as Internal>::Key,
    );

    /// Add an agent to the set of terminals
    /// 
    /// The system does not finish execusion until each of the agents in the set of terminals
    /// has been terminated. Once all the agents in the set of terminals is done executing,
    ///  an agent not in this set will be dropped regardless of whether it terminated or not.
    fn add_terminal(&mut self, key: <Self::Internal as Internal>::Key);
}
