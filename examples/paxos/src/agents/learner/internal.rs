use crate::agents::learner::LearnerInternal;
use crate::agents::*;
use core::panic;
use std::hash::Hash;
use std::{collections::VecDeque, fmt::Debug};
use system::internal::{Instruction, Internal};
use system::synchronous::{AgentCB, InChannel, OutChannels};

pub type Learner<T> = AgentCB<LearnerInternal<T>, AgentID, Message<T>>;
type Queue<T> = VecDeque<Instruction<AgentID, Message<T>>>;

impl<T> Internal for LearnerInternal<T>
where
    T: Clone + Send + Eq + Hash + Debug + 'static,
{
    type Message = Message<T>;
    type Key = AgentID;
    type Queue = Queue<T>;
    type Error = AgentInternalError;

    fn start(&mut self) -> Self::Queue {
        VecDeque::from(vec![Instruction::Get])
    }

    fn process_message(&mut self, message: Option<Message<T>>) -> Self::Queue {
        if let Some(msg) = message {
            self.parse_message(msg).unwrap();
        }
        if let Some(val) = &self.value {
            return VecDeque::from(vec![Instruction::Terminate(Message::Terminated(
                self.id,
                val.clone(),
            ))]);
        }
        VecDeque::from(vec![Instruction::Get])
    }

    fn new_incoming_key(&mut self, key: &Self::Key) {
        match key {
            AgentID::Acceptor(_) => self.num_of_acceptors += 1,
            _ => panic!("Invalid connection!"),
        }
    }

    fn new_outgoing_key(&mut self, _: &Self::Key) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_run() {
        let internal: LearnerInternal<String> = LearnerInternal::new(0);
        let mut learner = Learner::new(internal);
        let tx = learner.in_channel.tx();

        learner.internal.set_num_of_acceptors(4);

        let id = { AgentID::Acceptor };
        tx.send(Message::NewVote(id(0), 1, String::from("Hello")))
            .unwrap();
        tx.send(Message::NewVote(id(1), 1, String::from("Hello")))
            .unwrap();
        tx.send(Message::NewVote(id(2), 1, String::from("Hello")))
            .unwrap();

        learner.run().unwrap();
        assert_eq!(learner.internal.value, Some(String::from("Hello")));
    }

    #[test]
    fn test_run_threaded() {
        use std::thread;
        use std::time::Duration;

        let n_acc = 10;
        let internal: LearnerInternal<String> = LearnerInternal::new(0);
        let mut learner = Learner::new(internal);
        let tx = learner.in_channel.tx();

        learner.internal.set_num_of_acceptors(4);

        let id = { AgentID::Acceptor };

        let voters = n_acc;

        let handle = thread::spawn(move || {
            learner.run().unwrap();
            learner.internal.value
        });

        for i in 0..voters {
            let tx_current = tx.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(1));
                tx_current
                    .send(Message::NewVote(id(i), 1, String::from("Hello")))
                    .unwrap();
            });
        }
        let value = handle.join().unwrap();

        assert_eq!(value, Some(String::from("Hello")));
    }
}
