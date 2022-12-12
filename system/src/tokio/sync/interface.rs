use crate::internal::*;
use std::fmt::Debug;
use tokio;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    Light,
    Blocking,
    Heavy,
}

#[derive(Debug)]
pub enum Interface<I : Internal> {
    Light(LightCore<I>),
    Blocking(Core<I>),
    Heavy(Core<I>),
}

impl<I: Internal> Interface<I>
{
    pub fn new(
        internal: I,
        kind: AgentType,
        tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
        rx: mpsc::Receiver<Option<I::Message>>,
    ) -> Self {
        match kind {
            AgentType::Light => Interface::Light(LightCore::new(internal, tx_inst, rx)),
            AgentType::Blocking => Interface::Blocking(Core::new(internal, tx_inst, rx)),
            AgentType::Heavy => Interface::Heavy(Core::new(internal, tx_inst, rx)),
        }
    }
    pub fn new_incoming_key(&mut self, key: &I::Key) {
        match self {
            Interface::Light(core) => core.new_incoming_key(key),
            Interface::Blocking(core) => core.new_incoming_key(key),
            Interface::Heavy(core) => core.new_incoming_key(key),
        }
    }

    pub fn new_outgoing_key(&mut self, key: &I::Key) {
        match self {
            Interface::Light(core) => core.new_outgoing_key(key),
            Interface::Blocking(core) => core.new_outgoing_key(key),
            Interface::Heavy(core) => core.new_outgoing_key(key),
        }
    }
}

#[derive(Debug)]
pub struct LightCore<I : Internal> {
    core: I,
    rx: mpsc::Receiver<Option<I::Message>>,
    tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
}

#[derive(Debug)]
pub struct Core<I: Internal> {
    core: I,
    rx: mpsc::Receiver<Option<I::Message>>,
    tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
}

pub type SyncCoreError<I> =
    CoreError<<I as Internal>::Error, Instruction<<I as Internal>::Key, <I as Internal>::Message>>;

#[derive(Debug, Clone)]
pub enum CoreError<E, Q> {
    InternalError(E),
    SendError(Q),
}

impl<E, Q> From<mpsc::error::SendError<Q>> for CoreError<E, Q> {
    fn from(err: mpsc::error::SendError<Q>) -> Self {
        CoreError::SendError(err.0)
    }
}

impl<E, Q> CoreError<E, Q> {
    fn internal(err: E) -> Self {
        CoreError::InternalError(err)
    }
}

impl<I : Internal> LightCore<I> {
    fn new(
        internal: I,
        tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
        rx: mpsc::Receiver<Option<I::Message>>,
    ) -> Self {
        LightCore {
            core: internal,
            rx,
            tx_inst,
        }
    }

    fn new_incoming_key(&mut self, key: &I::Key) {
        self.core.new_incoming_key(key)
    }

    pub fn new_outgoing_key(&mut self, key: &I::Key) {
        self.core.new_outgoing_key(key)
    }

    pub async fn start(&mut self) -> Result<(), SyncCoreError<I>> {
        let mut instructions = self.core.start();

        while let Some(inst) = instructions.pop_front() {
            self.tx_inst.send(inst).await.ok();
        }
        Ok(())
    }

    pub async fn process_message(&mut self, message: Option<I::Message>) -> Result<(), SyncCoreError<I>> {
        let mut instructions = self.core.process_message(message);

        while let Some(inst) = instructions.pop_front() {
            self.tx_inst.send(inst).await.ok();
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), SyncCoreError<I>> {
        self.start().await?;

        while let Some(message) = self.rx.recv().await {
            self.process_message(message).await?;
        }
        Ok(())
    }
}

impl<I : Internal> Core<I>
where {
    fn new(
        internal: I,
        tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
        rx: mpsc::Receiver<Option<I::Message>>,
    ) -> Self {
        Core {
            core: internal,
            rx,
            tx_inst,
        }
    }
    fn new_incoming_key(&mut self, key: &I::Key) {
        self.core.new_incoming_key(key)
    }

    pub fn new_outgoing_key(&mut self, key: &I::Key) {
        self.core.new_outgoing_key(key)
    }

    pub fn start(&mut self) -> Result<(), SyncCoreError<I>> {
        let mut instructions = self.core.start();

        while let Some(inst) = instructions.pop_front() {
            self.tx_inst.blocking_send(inst)?;
        }
        Ok(())
    }

    pub fn process_message(&mut self, message: Option<I::Message>) -> Result<(), SyncCoreError<I>> {
        let mut instructions = self.core.process_message(message);

        while let Some(inst) = instructions.pop_front() {
            //println!("made it!");
            self.tx_inst.blocking_send(inst)?;
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), SyncCoreError<I>> {
        self.start()?;

        while let Some(message) = self.rx.blocking_recv() {
            self.process_message(message)?;
        }
        Ok(())
    }
}
