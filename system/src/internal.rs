//! Primitives for describing the internal logic of an actor in a distributed system 
//! To define an actor, it is enough to implement the [`ActorInternal`] trait.
//!
//! See the example in the main documentation. 

use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

/// Error returned by the Sender.
/// 
/// If a message failed to send, the error returns the message back. 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendError<T>(pub T);


/// The next state the actor can be in after doing a local operation. 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextState<T> {
    /// Wait for a message.
    Get,
    /// Wait for a message, but only for a certain duration of time.
    GetTimeout(Duration),
    /// Terminate, and return an optional termination message to be collected by the system.
    Terminate(Option<T>),
}

/// Sender interface for passing messages between actors. 
/// 
/// Actors will implement their operations with respect to a generic sender. Each 
/// implementation of the system will instatitate its own sender. Users can also implement thier own
/// senders. 
pub trait Sender: Debug + Send + Clone + 'static {
    /// The type of messages that are sent between actors
    type Message: Send + Clone + Debug + 'static;

    /// Identifier for an actor
    ///
    /// In the future, added possibly for a set of actors (currently not supported).
    type Key: Hash + Send + Copy + Debug + Eq + PartialEq;

    /// Send a message to the channel identified by [`key`].
    fn send(
        &mut self,
        key: &Self::Key,
        message: Self::Message,
    ) -> Result<(), SendError<(Self::Key, Self::Message)>>;
}


/// An interface for describing the internal operation of an agent.
///
/// An internal interface should contain the agent's local variables and the logic of the
/// agent's operation. The internal system interacts with external interfaces by sending
/// instructions and recieving messages.
///
/// The agent sends messages by invoking the send method of the sender. After every message recievied,
/// the agent can perform some internal operations and then wait for the next message, or wait for a
/// certain amount of time.
pub trait ActorInternal: Debug + Send + 'static {
    /// Messages that are sent between actors
    type Message: Debug + Send + Clone + Debug + 'static;
    /// Identifier for an actor
    ///
    /// In the future, added possibly for a set of actors (currently not supported).
    type Key: Hash + Send + Copy + Debug + Eq + PartialEq;

    /// The error type for an actor's internal system
    type Error: Send + Debug + From<SendError<(Self::Key, Self::Message)>>;

    /// Get an incoming channel from the system to an actor with identifier given by Key.
    ///
    /// An actor may or may not wish to save incoming keys in its local memory in
    /// order to be able to make sending instructions to the channel.
    fn new_incoming_key(&mut self, key: &Self::Key);
    /// Get an outgoing channel from the system to an actor with identifier given by [`key`].
    ///
    ///  An actor may or may not wish to save outgoing keys in its local memory in
    /// order to be able to make sending instructions to the channel.
    fn new_outgoing_key(&mut self, key: &Self::Key);

    /// Starting operations of the actor.
    ///
    /// Usually used to make all the steps before needing to wait for messages. If the startup
    /// exited succesfully, the actor can wait for a message using Get or GetTimeout.
    fn start<T: Sender<Key = Self::Key, Message = Self::Message>>(
        &mut self,
        tx: &mut T,
    ) -> Result<NextState<Self::Message>, Self::Error>;

    /// Process a potential message
    ///
    /// After sening a Get or GetTimeout commands, an actor will get Some(message)
    /// if waited for a message and a None if either the timeout has elapsed or
    /// the channel has disconnected.
    fn process_message<T: Sender<Key = Self::Key, Message = Self::Message>>(
        &mut self,
        message: Option<Self::Message>,
        tx: &mut T,
    ) -> Result<NextState<Self::Message>, Self::Error>;
}




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
    Terminate(Option<T>),
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