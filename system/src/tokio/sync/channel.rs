use std::collections::HashMap;
use std::hash::Hash;
use tokio::sync::mpsc;

pub type SendError<T> = mpsc::error::SendError<T>;

#[derive(Debug)]
pub struct Channels<K, M> {
    tx: mpsc::Sender<M>,
    pub rx: mpsc::Receiver<M>,
    pub out_channels: HashMap<K, mpsc::Sender<M>>,
}

impl<K, M> Channels<K, M>
where
    K: Eq + Hash + Copy,
{
    pub fn new(buffer: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer);
        Channels {
            tx,
            rx,
            out_channels: HashMap::new(),
        }
    }

    /// Insert an outgoing channel
    ///
    /// In case a channel is already associated with the key, it returns it.
    pub fn insert(&mut self, key: K, tx: mpsc::Sender<M>) -> Option<mpsc::Sender<M>> {
        self.out_channels.insert(key, tx)
    }

    /// Removes a channel associated with key and returns it.
    ///
    /// If no channel exists, returns none.
    pub fn remove(&mut self, key: &K) -> Option<mpsc::Sender<M>> {
        self.out_channels.remove(key)
    }

    pub async fn send(&self, key: &K, message: M) -> Result<(), SendError<M>> {
        self.out_channels.get(key).unwrap().send(message).await
    }

    pub fn get(&self, key: &K) -> Option<mpsc::Sender<M>> {
        self.out_channels.get(key).cloned()
    }

    pub async fn recv(&mut self) -> Option<M> {
        self.rx.recv().await
    }

    pub fn tx(&self) -> mpsc::Sender<M> {
        self.tx.clone()
    }
}
