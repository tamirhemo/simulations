use std::marker::PhantomData;

use super::channel::{Channels, SendError};
use super::interface::*;
use crate::internal::*;
use std::fmt::Debug;
use std::hash::Hash;
use tokio;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Agent<I, T, M, C> {
    tx: mpsc::Sender<T>,
    rx_inst: mpsc::Receiver<M>,
    pub channels: C,
    _phantom: PhantomData<I>,
}

#[derive(Debug)]
pub enum AgentError<E, Q, T> {
    InterfaceError(CoreError<E, Q>),
    SendError(SendError<T>),
    ExitedWithoutValue,
}

impl<E, Q, T> From<SendError<T>> for AgentError<E, Q, T> {
    fn from(e: SendError<T>) -> Self {
        AgentError::SendError(e)
    }
}

impl<E, Q, T> From<CoreError<E, Q>> for AgentError<E, Q, T> {
    fn from(err: CoreError<E, Q>) -> Self {
        AgentError::InterfaceError(err)
    }
}

pub type SyncAgent<I, K, T> = Agent<I, Option<T>, Instruction<K, T>, Channels<K, T>>;

impl<I, K, T> Agent<I, Option<T>, Instruction<K, T>, Channels<K, T>>
where
    T: Debug + 'static,
    K: Debug + Eq + Copy + Hash + 'static,
{
    pub fn new(
        internal: I,
        kind: AgentType,
        buffer: usize,
        internal_buffer: usize,
    ) -> (Self, Interface<I, Option<T>, Instruction<K, T>>)
    where
        I: Internal<Key = K, Message = T> + Send + Debug + 'static,
        T: Send + Debug + 'static,
        K: Send + Debug + 'static,
        I::Error: Debug + Send + 'static,
    {
        let (tx, rx) = mpsc::channel(buffer);
        let (tx_inst, rx_inst) = mpsc::channel(internal_buffer);
        (
            Agent {
                tx: tx,
                rx_inst: rx_inst,
                channels: Channels::new(buffer),
                _phantom: PhantomData,
            },
            Interface::new(internal, kind, tx_inst, rx),
        )
    }

    pub async fn run_command(
        &mut self,
        command: Instruction<K, T>,
    ) -> Result<Option<T>, AgentError<I::Error, Instruction<I::Key, I::Message>, I::Message>>
    where
        I: Internal<Key = K, Message = T> + Send + Debug + 'static,
        I::Error: Send + Debug + 'static,
    {
        match command {
            Instruction::Send(k, msg) => {
                let tx = self.channels.get(&k).unwrap();
                tx.send(msg).await.ok();
            }
            Instruction::Get => {
                let message = self.channels.recv().await;
                self.tx.send(message).await.ok();
            }
            Instruction::GetTimeout(timeout) => {
                let message = tokio::time::timeout(timeout, self.channels.recv())
                    .await
                    .ok()
                    .flatten();
                self.tx.send(message).await.ok();
            }
            Instruction::Terminate(msg) => return Ok(Some(msg)),
        };
        Ok(None)
    }

    pub async fn run(
        &mut self,
        termination: Option<mpsc::Sender<T>>,
    ) -> Result<(), AgentError<I::Error, Instruction<I::Key, I::Message>, I::Message>>
    where
        I: Internal<Key = K, Message = T> + Send + Debug + 'static,
        I::Error: Send + Debug + 'static,
    {
        while let Some(command) = self.rx_inst.recv().await {
            let return_value = self.run_command(command).await?;
            if let Some(msg) = return_value {
                if let Some(tx) = termination {
                    tx.send(msg).await.unwrap();
                }
                return Ok(());
            }
        }
        Err(AgentError::ExitedWithoutValue)
    }
}
