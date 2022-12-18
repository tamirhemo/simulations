use crate::internal::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

/// A generic interface for an Agent's incoming channel
pub trait InChannel: Clone + Send {
    type Message;
    type Sender;

    fn new() -> Self;

    fn tx(&self) -> Self::Sender;

    /// Blocking current thread and wait for a message
    fn recv(&self) -> Option<Self::Message>;

    /// Block thread to wait for message for a limited time
    fn recv_timeout(&self, timeout: Duration) -> Option<Self::Message>;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelError<T> {
    SendError(T),
    RecvTimeoutError(RecvTimeoutError),
    RecvError,
}

/// A generic interface for the functionality of Agent's outgoing channels
pub trait OutChannels: Debug + Clone + Send + 'static {
    type Message: Debug + Send + Clone + 'static;
    type Key: Debug + Send + Clone + Copy + Hash + Eq + 'static;
    type Sender;

    /// The empry interface without outgoing channels
    fn new() -> Self;

    /// Send a message in outgoing channel marked by key
    fn send(
        &self,
        key: &Self::Key,
        message: Self::Message,
    ) -> Result<(), SendError<(Self::Key, Self::Message)>>;

    /// Insert an outgoing channel
    ///
    /// In case a channel is already associated with the key, it returns it.
    fn insert(&mut self, key: Self::Key, tx: Self::Sender) -> Option<Self::Sender>;

    /// Removes a channel associated with key and returns it.
    ///
    /// If no channel exists, returns none.
    fn remove(&mut self, key: Self::Key) -> Option<Self::Sender>;
}

impl<S: OutChannels> Sender for S {
    type Key = S::Key;
    type Message = S::Message;

    fn send(
        &mut self,
        key: &Self::Key,
        message: Self::Message,
    ) -> Result<(), SendError<(Self::Key, Self::Message)>> {
        OutChannels::send(self, key, message)
    }
}

/// An error describing a closed channel
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct RecvError;

/// An error returned from the [`recv_timeout`] method.
///
/// [`recv_timeout`]: InChannel::recv_timeout
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RecvTimeoutError {
    Timeout,
    Disconnected,
}

impl<T> From<SendError<T>> for ChannelError<T> {
    fn from(err: SendError<T>) -> Self {
        ChannelError::SendError(err.0)
    }
}

