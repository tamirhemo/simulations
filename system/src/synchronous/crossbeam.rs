use super::actor::*;
use super::channel::*;
use super::system::SyncSystem;
use crate::internal::*;
use crossbeam_channel as cb;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct CrossbeamInterface<I: ActorInternal> {
    _marker: PhantomData<I>,
}

#[derive(Debug, Clone)]
pub struct OutChannelsCB<K, T> {
    pub ch_map: HashMap<K, cb::Sender<T>>,
}

pub type CrossbeamSystem<I> = SyncSystem<CrossbeamInterface<I>>;

impl<I: ActorInternal> ActorInterface for CrossbeamInterface<I> {
    type Message = I::Message;
    type Key = I::Key;
    type Error = I::Error;
    type Sender = cb::Sender<I::Message>;

    type InChannel = (cb::Sender<I::Message>, cb::Receiver<I::Message>);
    type OutChannels = OutChannelsCB<I::Key, I::Message>;
    type Internal = I;
}

impl<K, T> OutChannels for OutChannelsCB<K, T>
where
    K: Debug + Eq + Hash + Copy + Send + 'static,
    T: Debug + Clone + Send + 'static,
{
    type Message = T;
    type Key = K;
    type Sender = cb::Sender<T>;

    fn new() -> Self {
        OutChannelsCB {
            ch_map: HashMap::new(),
        }
    }
    
    fn send(&self, key: &K, message: T) -> Result<(), SendError<(K, T)>> {
        match self.ch_map.get(key).unwrap().send(message) {
            Ok(_) => Ok(()),
            Err(err) => Err(SendError((*key, err.0))),
        }
    }
    fn insert(&mut self, key: K, tx: cb::Sender<T>) -> Option<cb::Sender<T>> {
        self.ch_map.insert(key, tx)
    }
    fn remove(&mut self, key: Self::Key) -> Option<Self::Sender> {
        self.ch_map.remove(&key)
    }
}

impl<T> From<cb::SendError<T>> for SendError<T> {
    fn from(err: cb::SendError<T>) -> Self {
        SendError(err.0)
    }
}

impl<T: Clone + Send> InChannel for (cb::Sender<T>, cb::Receiver<T>) {
    type Message = T;
    type Sender = cb::Sender<T>;

    fn new() -> (cb::Sender<T>, cb::Receiver<T>) {
        cb::unbounded()
    }

    fn tx(&self) -> Self::Sender {
        self.0.clone()
    }

    fn recv(&self) -> Option<Self::Message> {
        self.1.recv().ok()
    }

    fn recv_timeout(&self, timeout: std::time::Duration) -> Option<Self::Message> {
        self.1.recv_timeout(timeout).ok()
    }
}
