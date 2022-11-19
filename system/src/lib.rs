//! An interace to generate simulations of distributed systems
//!realized as a set of agents and a set of message-passing channels.
//! The user only needs to write an implementation of each Agent's inner logic
//! and a function that sets up the initial conditions.
//!
//! The internal logic of an agent is expressed by implementing the [`Internal`] trait.
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
//! **Note**: The reason we did not abstract away the properties of a system in a trait is
//! that including async methods in trait is an unstable fearture in Rust.
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
//! To realize CycleInternal as an agent, we need to implement the [`Internal`] trait.
//!
//! ```
//! impl Internal for CycleInternal {
//!     type Message = usize;
//!     type Error = ();
//!     type Key = usize;
//!     type Queue = VecDeque<Instruction<usize, usize>>;
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
//!     fn start(&mut self) -> Self::Queue {
//!         let mut instructions = VecDeque::new();
//!         if self.starter {
//!             let out = self.output_key.unwrap();
//!             instructions.push_back(Instruction::Send(out, 0));
//!         }
//!         instructions.push_back(Instruction::Get);
//!
//!         instructions
//!     }
//!
//!    fn process_message(&mut self, message: Option<Self::Message>) -> Self::Queue {
//!        assert!(message.is_some());
//!        let value = message.unwrap();
//!    
//!        let out = self.output_key.unwrap();
//!        VecDeque::from(vec![Instruction::Send(out, value + 1), Instruction::Terminate(value+1)])
//!    }
//! }
//!```
//!

//! </details>
//!
//! We can now initiate and start the system as follows:
//!
//!```
//! # use synchronous::crossbeam::System;
//!
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
//! assert_eq!(values.get(&0), Some(&n));
//!```
//!

pub mod internal;
pub mod synchronous;
pub mod tokio;

pub use internal::{Instruction, Internal};
