use crate::internal::*;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::Hash;
use tokio;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorType {
    Light,
    Blocking,
    Heavy,
}

pub trait TokioInternal: Send + 'static {
    type Message: Send + Clone + Debug + 'static;
    type Key: Hash + Send + Copy + Debug + Eq + PartialEq;

    type Error: Send + Debug;

    fn new_incoming_key(&mut self, key: &Self::Key);
    fn new_outgoing_key(&mut self, key: &Self::Key);

    fn start_light(
        &mut self,
        tx: &mut VecDeque<Instruction<Self::Key, Self::Message>>,
    ) -> Result<NextState<Self::Message>, Self::Error>;

    fn process_message_light(
        &mut self,
        message: Option<Self::Message>,
        tx: &mut VecDeque<Instruction<Self::Key, Self::Message>>,
    ) -> Result<NextState<Self::Message>, Self::Error>;

    fn start_blocking(
        &mut self,
        tx: &mut mpsc::Sender<Instruction<Self::Key, Self::Message>>,
    ) -> Result<NextState<Self::Message>, Self::Error>;

    fn process_message_blocking(
        &mut self,
        message: Option<Self::Message>,
        tx: &mut mpsc::Sender<Instruction<Self::Key, Self::Message>>,
    ) -> Result<NextState<Self::Message>, Self::Error>;
}

/// A core containing an actor who performs blocking code

#[derive(Debug)]
pub struct LightCore<I: TokioInternal> {
    core: I,
    rx: mpsc::Receiver<Option<I::Message>>,
    tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
}

/// A core containing an actor who performs blocking code
#[derive(Debug)]
pub struct HeavyCore<I: TokioInternal> {
    core: I,
    rx: mpsc::Receiver<Option<I::Message>>,
    tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
}

#[derive(Debug)]
pub enum ActorCore<I: TokioInternal> {
    Light(LightCore<I>),
    Blocking(HeavyCore<I>),
    Heavy(HeavyCore<I>),
}

impl<K, T> From<mpsc::error::SendError<Instruction<K, T>>> for SendError<(K, T)> {
    fn from(err: mpsc::error::SendError<Instruction<K, T>>) -> Self {
        if let Instruction::Send(key, message) = err.0 {
            return SendError((key, message));
        }
        panic!("Not a message!")
    }
}

impl<K, T> Sender for mpsc::Sender<Instruction<K, T>>
where
    K: Debug + Send + 'static + Clone + Copy + Hash + Eq + PartialEq,
    T: Debug + Send + 'static + Clone,
{
    type Key = K;
    type Message = T;

    fn send(
        &mut self,
        key: &Self::Key,
        message: Self::Message,
    ) -> Result<(), SendError<(Self::Key, Self::Message)>> {
        Ok(self.blocking_send(Instruction::Send(*key, message))?)
    }
}

impl<K, T> Sender for VecDeque<Instruction<K, T>>
where
    K: Debug + Send + 'static + Clone + Copy + Hash + Eq + PartialEq,
    T: Debug + Send + 'static + Clone,
{
    type Key = K;
    type Message = T;

    fn send(
        &mut self,
        key: &Self::Key,
        message: Self::Message,
    ) -> Result<(), SendError<(Self::Key, Self::Message)>> {
        self.push_back(Instruction::Send(*key, message));
        Ok(())
    }
}

impl<I: ActorInternal> TokioInternal for I {
    type Message = I::Message;
    type Key = I::Key;
    type Error = I::Error;

    fn new_incoming_key(&mut self, key: &Self::Key) {
        self.new_incoming_key(key)
    }
    fn new_outgoing_key(&mut self, key: &Self::Key) {
        self.new_outgoing_key(key)
    }

    fn start_light(
        &mut self,
        tx: &mut VecDeque<Instruction<Self::Key, Self::Message>>,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        self.start(tx)
    }

    fn process_message_light(
        &mut self,
        message: Option<Self::Message>,
        tx: &mut VecDeque<Instruction<Self::Key, Self::Message>>,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        self.process_message(message, tx)
    }

    fn start_blocking(
        &mut self,
        tx: &mut mpsc::Sender<Instruction<Self::Key, Self::Message>>,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        self.start(tx)
    }

    fn process_message_blocking(
        &mut self,
        message: Option<Self::Message>,
        tx: &mut mpsc::Sender<Instruction<Self::Key, Self::Message>>,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        self.process_message(message, tx)
    }
}

impl<I: TokioInternal> ActorCore<I> {
    pub fn new(
        internal: I,
        kind: ActorType,
        tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
        rx: mpsc::Receiver<Option<I::Message>>,
    ) -> Self {
        match kind {
            ActorType::Light => ActorCore::Light(LightCore::new(internal, tx_inst, rx)),
            ActorType::Blocking => ActorCore::Blocking(HeavyCore::new(internal, tx_inst, rx)),
            ActorType::Heavy => ActorCore::Heavy(HeavyCore::new(internal, tx_inst, rx)),
        }
    }
    pub fn new_incoming_key(&mut self, key: &I::Key) {
        match self {
            ActorCore::Light(core) => core.new_incoming_key(key.into()),
            ActorCore::Blocking(core) => core.new_incoming_key(key.into()),
            ActorCore::Heavy(core) => core.new_incoming_key(key.into()),
        }
    }

    pub fn new_outgoing_key(&mut self, key: &I::Key) {
        match self {
            ActorCore::Light(core) => core.new_outgoing_key(key.into()),
            ActorCore::Blocking(core) => core.new_outgoing_key(key.into()),
            ActorCore::Heavy(core) => core.new_outgoing_key(key.into()),
        }
    }
}

impl<I: TokioInternal> LightCore<I> {
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

    pub async fn start(&mut self) -> Result<(), I::Error> {
        let mut instructions = VecDeque::new();

        let next_state = self.core.start_light(&mut instructions)?;
        instructions.push_back(next_state.into());

        while let Some(inst) = instructions.pop_front() {
            self.tx_inst.send(inst).await.ok();
        }
        Ok(())
    }

    pub async fn process_message(&mut self, message: Option<I::Message>) -> Result<(), I::Error> {
        let mut instructions = VecDeque::new();

        let next_state = self
            .core
            .process_message_light(message, &mut instructions)?;
        instructions.push_back(next_state.into());

        while let Some(inst) = instructions.pop_front() {
            self.tx_inst.send(inst).await.ok();
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), I::Error> {
        self.start().await?;

        while let Some(message) = self.rx.recv().await {
            self.process_message(message).await?;
        }
        Ok(())
    }
}

impl<I: TokioInternal> HeavyCore<I> {
    fn new(
        internal: I,
        tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
        rx: mpsc::Receiver<Option<I::Message>>,
    ) -> Self {
        HeavyCore {
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

    pub fn start(&mut self) -> Result<(), I::Error> {
        let next_state = self.core.start_blocking(&mut self.tx_inst);

        match next_state {
            Ok(state) => self.tx_inst.blocking_send(state.into()),
            Err(e) => self
                .tx_inst
                .blocking_send(NextState::Terminate(None).into()),
        };
        Ok(())
    }

    //TODO: Error handeling
    pub fn process_message(&mut self, message: Option<I::Message>) -> Result<(), I::Error> {
        let next_state = self
            .core
            .process_message_blocking(message, &mut self.tx_inst);

        match next_state {
            Ok(state) => self.tx_inst.blocking_send(state.into()),
            Err(e) => self
                .tx_inst
                .blocking_send(NextState::Terminate(None).into()),
        };
        Ok(())
    }

    //TODO: Error handeling
    pub fn run(&mut self) -> Result<(), I::Error> {
        self.start()?;

        while let Some(message) = self.rx.blocking_recv() {
            self.process_message(message)?;
        }
        Ok(())
    }
}
