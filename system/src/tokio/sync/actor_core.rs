use crate::internal::*;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::Hash;
use tokio;
use tokio::sync::mpsc;


/// A type signifying the type of operations happening in the actor's internal code.
/// 
/// This is determined by the user and is used by the tokio based implementation to decide 
/// where to place the internal operations within asynchronous code. The actor can be of the following types:
/// * Light - the actor's internal code is not blocking and can be run within an asynchronous function.
/// * Blocking - the actor's internal code is blocking but not cpu heavy so it can be places within tokio::spawn_blocking task.
/// * Heavy - the actor internal code is performing cpu heavy operations. A dedicated std::thread will be spawned for performing the internal operations of the actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorType {
    Light,
    Blocking,
    Heavy,
}

/// Functionalities of actor internal specialized to a system based on the tokio runtime
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

/// A core containing an actor who performs synchronous code which is not blocking 
#[derive(Debug)]
pub struct LightCore<I: TokioInternal> {
    core: I,
    rx: mpsc::Receiver<Option<I::Message>>,
    tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
}

/// A core containing an actor who performs synchrnous code that might be blocking. 
#[derive(Debug)]
pub struct HeavyCore<I: TokioInternal> {
    core: I,
    rx: mpsc::Receiver<Option<I::Message>>,
    tx_inst: mpsc::Sender<Instruction<I::Key, I::Message>>,
}


/// A type for the internal workings of an actor used by system based on the tokio runtime. 
/// 
/// The actor can be of the following types (determined by the user)
/// Light - the actor's internal code is not blocking and can be run within an asynchronous function.
///
#[derive(Debug)]
pub enum ActorCore<I: TokioInternal> {
    Light(LightCore<I>),
    Blocking(HeavyCore<I>),
    Heavy(HeavyCore<I>),
}


#[derive(Debug)]
pub enum CoreError<I: TokioInternal> {
    InternalError(I::Error),
    InstructionChannelError(Instruction<I::Key, I::Message>),
}


impl<I : TokioInternal> From<mpsc::error::SendError<Instruction<I::Key, I::Message>>> for CoreError<I> {
    fn from(err: mpsc::error::SendError<Instruction<I::Key, I::Message>>) -> Self {
        CoreError::InstructionChannelError(err.0)
    }
}

impl<I: TokioInternal> CoreError<I> {

    #[inline]
    fn from_internal(err: I::Error) -> Self {
        CoreError::InternalError(err)
    }
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

    pub async fn start(&mut self) -> Result<(), CoreError<I>> {
        let mut instructions = VecDeque::new();

        let next_state = 
            self.core.start_light(&mut instructions)
            .map_err(CoreError::from_internal)?;
        instructions.push_back(next_state.into());

        while let Some(inst) = instructions.pop_front() {
            self.tx_inst.send(inst).await?;
        }
        Ok(())
    }

    pub async fn process_message(&mut self, message: Option<I::Message>) -> Result<(), CoreError<I>> {
        let mut instructions = VecDeque::new();

        let next_state = self
            .core
            .process_message_light(message, &mut instructions)
            .map_err(CoreError::from_internal)?;
        instructions.push_back(next_state.into());

        while let Some(inst) = instructions.pop_front() {
            self.tx_inst.send(inst).await?;
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), CoreError<I>> {
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

    pub fn start(&mut self) -> Result<(), CoreError<I>> {
        let next_state = 
            self.core.start_blocking(&mut self.tx_inst)
            .map_err(CoreError::from_internal)?;
        
        self.tx_inst.blocking_send(next_state.into())?;
        Ok(())
    }

    pub fn process_message(&mut self, message: Option<I::Message>) -> Result<(), CoreError<I>> {
        let next_state = self
            .core
            .process_message_blocking(message, &mut self.tx_inst)
            .map_err(CoreError::from_internal)?;

        self.tx_inst.blocking_send(next_state.into())?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), CoreError<I>> {
        self.start()?;

        while let Some(message) = self.rx.blocking_recv() {
            self.process_message(message)?;
        }
        Ok(())
    }
}
