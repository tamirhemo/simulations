use std::collections::VecDeque;
use std::time::Duration;

/// Instructions an agent internal system can output to its channel interface
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction<K, T> {
    Send(K, T),
    Terminate(T),
    Get,
    GetTimeout(Duration),
}

/// Basic functionalities for a queue of instructions
pub trait InstructionQueue {
    type Instruction;

    fn pop_front(&mut self) -> Option<Self::Instruction>;
    fn append(&mut self, other: &mut Self);
}
/// An interface for the internal operation of an agent.
///
/// An internal interface should contain the agent's local variables and the logic of the
/// agent's operation. The internal system interacts with external interfaces by sending
/// instructions and recieving messages.
pub trait Internal {
    type Message;
    type Key;
    type Error;
    type Queue: InstructionQueue<Instruction = Instruction<Self::Key, Self::Message>>;

    fn new_incoming_key(&mut self, key: &Self::Key);
    fn new_outgoing_key(&mut self, key: &Self::Key);

    fn start(&mut self) -> Self::Queue;
    fn process_message(&mut self, message: Option<Self::Message>) -> Self::Queue;
}

// Convenient implementation for outside use
impl<K, T> InstructionQueue for VecDeque<Instruction<K, T>> {
    type Instruction = Instruction<K, T>;

    fn pop_front(&mut self) -> Option<Self::Instruction> {
        self.pop_front()
    }

    fn append(&mut self, other: &mut Self) {
        self.append(other)
    }
}
