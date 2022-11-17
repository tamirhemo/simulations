use std::time::Duration;

/// A generic interface for an Agent's incoming channel
pub trait InChannel {
    type Message;
    type Sender;

    fn new() -> Self;

    fn tx(&self) -> Self::Sender;

    /// Blocking current thread and wait for a message
    fn recv(&self) -> Result<Self::Message, RecvError>;

    /// Block thread to wait for message for a limited time
    fn recv_timeout(&self, timeout: Duration) -> Result<Self::Message, RecvTimeoutError>;
}

impl From<RecvError> for RecvTimeoutError {
    fn from(_: RecvError) -> Self {
        RecvTimeoutError::Disconnected
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelError<T> {
    SendError(T),
    RecvTimeoutError(RecvTimeoutError),
    RecvError,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SendError<T>(pub T);

/// A generic interface for the functionality of Agent's outgoing channels
pub trait OutChannels {
    type Message;
    type Sender;
    type Key;

    /// The empry interface without outgoing channels
    fn new() -> Self;

    /// Send a message in outgoing channel marked by key
    fn send(&self, key: Self::Key, message: Self::Message) -> Result<(), SendError<Self::Message>>;

    /// Insert an outgoing channel
    ///
    /// In case a channel is already associated with the key, it returns it.
    fn insert(&mut self, key: Self::Key, tx: Self::Sender) -> Option<Self::Sender>;

    /// Removes a channel associated with key and returns it.
    ///
    /// If no channel exists, returns none.
    fn remove(&mut self, key: Self::Key) -> Option<Self::Sender>;
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

impl<T> From<RecvError> for ChannelError<T> {
    fn from(_: RecvError) -> Self {
        ChannelError::RecvError
    }
}

impl<T> From<RecvTimeoutError> for ChannelError<T> {
    fn from(err: RecvTimeoutError) -> Self {
        ChannelError::RecvTimeoutError(err)
    }
}
