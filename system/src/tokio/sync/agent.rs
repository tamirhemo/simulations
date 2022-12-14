use std::marker::PhantomData;

use super::channel::{Channels, SendError};
use super::agent_core::*;
use crate::internal::*;
use std::fmt::Debug;
use tokio;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Agent<I: Internal, C> {
    core : AgentCore<I>,
    interface : AgentInterface<I, C>,
}

impl<I: Internal> Agent<I, Channels<I::Key, I::Message>> {
    pub fn new(
        internal: I,
        kind: AgentType,
        buffer: usize,
        internal_buffer: usize,
    ) -> Self {
        let (tx, rx) = mpsc::channel(buffer);
        let (tx_inst, rx_inst) = mpsc::channel(internal_buffer);
        Agent { core: AgentCore::new(internal, kind, tx_inst, rx), 
            interface: AgentInterface::new(tx,rx_inst, buffer) ,
             }
    }

    pub fn split(self) -> (AgentCore<I>, AgentInterface<I, Channels<I::Key, I::Message>>) {
        (self.core, self.interface)
    }

    pub fn tx_channel(&self) -> mpsc::Sender<I::Message> {
        self.interface.channels.tx()
    }

    pub fn insert_outgoing_channel(&mut self, key: I::Key, tx: mpsc::Sender<I::Message>) -> Option<mpsc::Sender<I::Message>> {
        self.interface.channels.out_channels.insert(key, tx)
    }

    pub fn new_outgoing_key(&mut self, key: &I::Key) {
        self.core.new_outgoing_key(key)
    }

    pub fn new_incoming_key(&mut self, key: &I::Key) {
        self.core.new_incoming_key(key)
    }
}

#[derive(Debug)]
pub struct AgentInterface<I: Internal, C> {
    tx: mpsc::Sender<Option<I::Message>>,
    rx_inst: mpsc::Receiver<Instruction<I::Key, I::Message>>,
    pub channels: C,
    _phantom: PhantomData<I>,
}

#[derive(Debug)]
pub enum AgentError<I: Internal> {
    InterfaceError(SyncCoreError<I>),
    SendError(SendError<I::Message>),
    ExitedWithoutValue,
}

impl<I: Internal> From<SendError<I::Message>> for AgentError<I> {
    fn from(e: SendError<I::Message>) -> Self {
        AgentError::SendError(e)
    }
}

impl<I: Internal> From<SyncCoreError<I>> for AgentError<I> {
    fn from(err: SyncCoreError<I>) -> Self {
        AgentError::InterfaceError(err)
    }
}

//pub type SyncAgent<I, K, T> = Agent<I, Option<T>, Instruction<K, T>, Channels<K, T>>;

impl<I: Internal> AgentInterface<I, Channels<I::Key, I::Message>> {
    pub fn new(
        tx : mpsc::Sender<Option<<I as Internal>::Message>>,
        rx_inst : mpsc::Receiver<Instruction<<I as Internal>::Key, <I as Internal>::Message>>,
        buffer: usize,
    ) -> Self {
            AgentInterface {
                tx,
                rx_inst,
                channels: Channels::new(buffer),
                _phantom: PhantomData,
            }
            //Interface::new(internal, kind, tx_inst, rx),
    }

    pub async fn run_command(
        &mut self,
        command: Instruction<I::Key, I::Message>,
    ) -> Result<Option<I::Message>, AgentError<I>> {
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
        termination: Option<mpsc::Sender<I::Message>>,
    ) -> Result<(), AgentError<I>> {
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
