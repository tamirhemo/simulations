//! A primitive for describing the internal logic of an agent in a distributed system.
//!
//! To define an agent the user needs to implement the [`Internal`] trait.
//!

use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

/// Instructions that an agent's internal system can give its incoming-outgoing channel interface.
///  
/// This describes the agent's interaction with the outside world.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction<K, T> {
    ///Send a message of type `T` along a channel identified by a key of type `K`.
    Send(K, T),
    /// Wait for a message.
    Get,
    /// Wait for a message, but only up for the duration of timeout.
    GetTimeout(Duration),
    /// Return a termination message to be collected by the system.
    Terminate(T),
}

/// Basic functionalities for a queue of instructions
pub trait InstructionQueue {
    type Message: Send + Clone + Debug + 'static;
    type Key: Hash + Send + Copy + Debug + Eq + PartialEq;

    fn pop_front(&mut self) -> Option<Instruction<Self::Key, Self::Message>>;
    fn push_back(&mut self, instruction: Instruction<Self::Key, Self::Message>);
    fn append(&mut self, other: &mut Self);
}
/// An interface for describing the internal operation of an agent.
///
/// An internal interface should contain the agent's local variables and the logic of the
/// agent's operation. The internal system interacts with external interfaces by sending
/// instructions and recieving messages.
///
///
pub trait Internal: Send + 'static {
    type Message: Send + Clone + Debug + 'static;
    type Key: Hash + Send + Copy + Debug + Eq + PartialEq;
    type Error: Send + Debug;
    type Queue: Send + InstructionQueue<Key = Self::Key, Message = Self::Message>;

    /// Get an incoming channel from the system to an agent with identifier given by Key.
    ///
    /// An agent may or may not wish to save incoming keys in its local memory in
    /// order to be able to make sending instructions to the channel.
    fn new_incoming_key(&mut self, key: &Self::Key);
    /// Get an outgoing channel from the system to an agent with identifier given by [`key`].
    ///
    ///  An agent may or may not wish to save outgoing keys in its local memory in
    /// order to be able to make sending instructions to the channel.
    fn new_outgoing_key(&mut self, key: &Self::Key);

    /// Starting operations of the agent.
    ///
    /// Usually used to make all the steps before needing to wait for messages and then
    /// sending a Get or GetTimeout command
    fn start(&mut self) -> Self::Queue;

    /// Process a potential message
    ///
    /// After sening a Get or GetTimeout commands, an agent will get Some(message)
    /// if waited for a message and a None if either the timeout has elapsed or
    /// the channel has disconnected.
    fn process_message(&mut self, message: Option<Self::Message>) -> Self::Queue;
}

// Convenient implementation for outside use
impl<K, T> InstructionQueue for VecDeque<Instruction<K, T>>
where
    K: Hash + Send + Copy + Debug + Eq + PartialEq,
    T: Send + Clone + Debug + 'static,
{
    type Key = K;
    type Message = T;

    fn pop_front(&mut self) -> Option<Instruction<K, T>> {
        self.pop_front()
    }

    fn push_back(&mut self, instruction: Instruction<K, T>) {
        self.push_back(instruction)
    }

    fn append(&mut self, other: &mut Self) {
        self.append(other)
    }
}

pub enum NextState<T> {
    /// Wait for a message.
    Get,
    /// Wait for a message, but only up for the duration of timeout.
    GetTimeout(Duration),
    /// Return a termination message to be collected by the system.
    Terminate(T),
}

impl<K, T> From<NextState<T>> for Instruction<K, T> {
    fn from(state: NextState<T>) -> Self {
        match state {
            NextState::Get => Instruction::Get,
            NextState::GetTimeout(t) => Instruction::GetTimeout(t),
            NextState::Terminate(message) => Instruction::Terminate(message),
        }
    }
}

pub struct Sender<'a, K, T>(&'a mut VecDeque<Instruction<K, T>>);
#[derive(Debug, Clone, PartialEq)]
pub struct SendError<T>(pub T);

impl<'a, K, T> Sender<'a, K, T> {
    pub fn send(&mut self, key: K, message: T) -> Result<(), SendError<(K, T)>> {
        self.0.push_back(Instruction::Send(key, message));
        Ok(())
    }
}

pub trait AgentInternal: Send + 'static {
    type Message: Send + Clone + Debug + 'static;
    type Key: Hash + Send + Copy + Debug + Eq + PartialEq;
    type Error: Send + Debug;

    /// Get an incoming channel from the system to an agent with identifier given by Key.
    ///
    /// An agent may or may not wish to save incoming keys in its local memory in
    /// order to be able to make sending instructions to the channel.
    fn new_incoming_key(&mut self, key: &Self::Key);
    /// Get an outgoing channel from the system to an agent with identifier given by [`key`].
    ///
    ///  An agent may or may not wish to save outgoing keys in its local memory in
    /// order to be able to make sending instructions to the channel.
    fn new_outgoing_key(&mut self, key: &Self::Key);

    /// Starting operations of the agent.
    ///
    /// Usually used to make all the steps before needing to wait for messages and then
    /// sending a Get or GetTimeout command
    fn start(
        &mut self,
        tx: &mut Sender<Self::Key, Self::Message>,
    ) -> Result<NextState<Self::Message>, Self::Error>;

    /// Process a potential message
    ///
    /// After sening a Get or GetTimeout commands, an agent will get Some(message)
    /// if waited for a message and a None if either the timeout has elapsed or
    /// the channel has disconnected.
    fn process_message(
        &mut self,
        message: Option<Self::Message>,
        tx: &mut Sender<Self::Key, Self::Message>,
    ) -> Result<NextState<Self::Message>, Self::Error>;
}

impl<T: AgentInternal> Internal for T {
    type Message = T::Message;
    type Key = T::Key;
    type Error = T::Error;
    type Queue = VecDeque<Instruction<T::Key, T::Message>>;

    fn new_incoming_key(&mut self, key: &Self::Key) {
        self.new_incoming_key(key)
    }

    fn new_outgoing_key(&mut self, key: &Self::Key) {
        self.new_outgoing_key(key)
    }

    fn start(&mut self) -> Self::Queue {
        let mut instructions = Self::Queue::new();
        let mut tx = Sender {
            0: &mut instructions,
        };

        let next_state = self.start(&mut tx).unwrap();
        instructions.push_back(next_state.into());

        instructions
    }

    fn process_message(&mut self, message: Option<Self::Message>) -> Self::Queue {
        let mut instructions = Self::Queue::new();
        let mut tx = Sender {
            0: &mut instructions,
        };

        let next_state = self.process_message(message, &mut tx).unwrap();
        instructions.push_back(next_state.into());

        instructions
    }
}
