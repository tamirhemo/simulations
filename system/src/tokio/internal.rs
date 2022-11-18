use crate::synchronous;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::mpsc;

pub type Instruction<K, T> = synchronous::Instruction<K, T>;

/// An interface for the internal operation of an agent.
///
/// An internal interface should contain the agent's local variables and the logic of the
/// agent's operation. The internal system interacts with external interfaces by sending
/// instructions and recieving messages, in this case through a channel.
///
pub trait SyncInternal {
    type Message;
    type Key;
    type Error;

    fn new_incoming_key(&mut self, key: &Self::Key);
    fn new_outgoing_key(&mut self, key: &Self::Key);

    fn start(
        &mut self,
        tx: mpsc::Sender<Instruction<Self::Key, Self::Message>>,
    ) -> Result<(), Self::Error>;

    fn process_message(
        &mut self,
        message: Option<Self::Message>,
        tx: mpsc::Sender<Instruction<Self::Key, Self::Message>>,
    ) -> Result<(), Self::Error>;
}

/// Basic functionalities for a queue of instructions
pub trait InstructionQueue {
    type Instruction;

    fn pop_front(&mut self) -> Option<Self::Instruction>;
    fn append(&mut self, other: &mut Self);
}

impl<T> InstructionQueue for VecDeque<T> {
    type Instruction = T;

    fn pop_front(&mut self) -> Option<Self::Instruction> {
        self.pop_front()
    }
    fn append(&mut self, other: &mut Self) {
        self.append(other)
    }
}

pub trait SyncInternalQueue {
    type Message;
    type Key;
    type Error;
    type Queue: InstructionQueue<Instruction = Instruction<Self::Key, Self::Message>>;

    fn new_incoming_key(&mut self, key: &Self::Key);
    fn new_outgoing_key(&mut self, key: &Self::Key);

    fn start(&mut self) -> Self::Queue;
    fn process_message(&mut self, message: Option<Self::Message>) -> Self::Queue;
}

impl<T> SyncInternal for T
where
    T: SyncInternalQueue + 'static + std::fmt::Debug,
    T::Key: std::fmt::Debug,
    T::Message: std::fmt::Debug,
{
    type Message = T::Message;
    type Key = T::Key;
    type Error = T::Error;

    fn new_incoming_key(&mut self, key: &Self::Key) {
        self.new_incoming_key(key)
    }

    fn new_outgoing_key(&mut self, key: &Self::Key) {
        self.new_outgoing_key(key)
    }

    fn start(
        &mut self,
        tx: mpsc::Sender<Instruction<Self::Key, Self::Message>>,
    ) -> Result<(), Self::Error> {
        let mut instructions = self.start();

        while let Some(command) = instructions.pop_front() {
            tx.blocking_send(command).unwrap();
        }
        Ok(())
    }

    fn process_message(
        &mut self,
        message: Option<Self::Message>,
        tx: mpsc::Sender<Instruction<Self::Key, Self::Message>>,
    ) -> Result<(), Self::Error> {
        let mut instructions = self.process_message(message);

        while let Some(command) = instructions.pop_front() {
            tx.blocking_send(command).unwrap();
        }
        Ok(())
    }
}

impl<T> SyncInternalQueue for T
where
    T: synchronous::Internal,
    T::Queue: InstructionQueue<Instruction = Instruction<T::Key, T::Message>>,
{
    type Message = T::Message;
    type Key = T::Key;
    type Error = T::Error;
    type Queue = T::Queue;

    fn new_incoming_key(&mut self, key: &Self::Key) {
        self.new_incoming_key(key)
    }
    fn new_outgoing_key(&mut self, key: &Self::Key) {
        self.new_outgoing_key(key)
    }

    fn start(&mut self) -> Self::Queue {
        self.start()
    }

    fn process_message(&mut self, message: Option<Self::Message>) -> Self::Queue {
        self.process_message(message)
    }
}
