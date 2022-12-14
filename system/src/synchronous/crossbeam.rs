use super::system;
use super::{agent, channel};
use crate::internal::Internal;
use channel::*;
use crossbeam_channel as cb;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

type Channel<T> = (cb::Sender<T>, cb::Receiver<T>);
pub type AgentCB<I, K, T> = agent::Agent<I, OutChannelsCB<K, T>, Channel<T>>;

pub type CrossbeamSystem<I> = system::SyncSystem<
    <I as Internal>::Key,
    AgentCB<I, <I as Internal>::Key, <I as Internal>::Message>,
>;

#[derive(Debug, Clone)]
pub struct OutChannelsCB<K, T> {
    pub ch_map: HashMap<K, cb::Sender<T>>,
}

impl<K: Eq + Hash, T: fmt::Debug> OutChannels for OutChannelsCB<K, T> {
    type Message = T;
    type Key = K;
    type Sender = cb::Sender<T>;

    fn new() -> Self {
        OutChannelsCB {
            ch_map: HashMap::new(),
        }
    }
    fn send(&self, key: K, message: T) -> Result<(), channel::SendError<T>> {
        Ok(self.ch_map.get(&key).unwrap().send(message)?)
    }
    fn insert(&mut self, key: K, tx: cb::Sender<T>) -> Option<cb::Sender<T>> {
        self.ch_map.insert(key, tx)
    }
    fn remove(&mut self, key: Self::Key) -> Option<Self::Sender> {
        self.ch_map.remove(&key)
    }
}

impl<T> From<cb::SendError<T>> for channel::SendError<T> {
    fn from(err: cb::SendError<T>) -> Self {
        channel::SendError(err.0)
    }
}

impl<T> InChannel for (cb::Sender<T>, cb::Receiver<T>) {
    type Message = T;
    type Sender = cb::Sender<T>;

    fn new() -> (cb::Sender<T>, cb::Receiver<T>) {
        cb::unbounded()
    }

    fn tx(&self) -> Self::Sender {
        self.0.clone()
    }

    fn recv(&self) -> Result<Self::Message, channel::RecvError> {
        Ok(self.1.recv()?)
    }

    fn recv_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<Self::Message, RecvTimeoutError> {
        Ok(self.1.recv_timeout(timeout)?)
    }
}

impl From<cb::RecvError> for RecvError {
    fn from(_: cb::RecvError) -> Self {
        RecvError {}
    }
}

impl From<cb::RecvTimeoutError> for RecvTimeoutError {
    fn from(err: cb::RecvTimeoutError) -> Self {
        match err {
            cb::RecvTimeoutError::Timeout => RecvTimeoutError::Timeout,
            cb::RecvTimeoutError::Disconnected => RecvTimeoutError::Disconnected,
        }
    }
}
