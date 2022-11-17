use crate::agents::acceptor::AcceptorInternal;
use crate::agents::*;
use std::{collections::VecDeque, fmt::Debug};
use system::mpsc::{AgentCB, InChannel, Instruction, Internal, OutChannels};

pub type Acceptor<T> = AgentCB<AcceptorInternal<T>, AgentID, Message<T>>;

type Queue<T> = VecDeque<Instruction<AgentID, Message<T>>>;

impl<T: Clone + Eq + Debug> Internal for AcceptorInternal<T> {
    type Message = Message<T>;
    type Key = AgentID;
    type Queue = Queue<T>;
    type Error = AgentInternalError;

    fn new_incoming_key(&mut self, _: &Self::Key) {}

    fn new_outgoing_key(&mut self, key: &Self::Key) {
        match key {
            AgentID::Learner(_) => self.learners.insert(*key),
            _ => false,
        };
    }

    fn start(&mut self) -> Self::Queue {
        VecDeque::from(vec![Instruction::Get])
    }

    fn process_message(&mut self, message: Option<Message<T>>) -> Self::Queue {
        if let Some(msg) = message {
            let mut instructions = match msg {
                Message::NewTime(ts, id) => {
                    self.proposers.insert(id);
                    if let Some(m) = self.parse_new_time(ts) {
                        VecDeque::from(vec![Instruction::Send(id, m)])
                    } else {
                        VecDeque::new()
                    }
                }

                Message::Proposal(ts, value, id) => {
                    assert!(self.proposers.contains(&id));
                    if self.parse_proposal(ts, value) {
                        let vote = self.make_vote().unwrap();
                        self.learners
                            .iter()
                            .map(|id| Instruction::Send(*id, vote.clone()))
                            .collect()
                    } else {
                        VecDeque::new()
                    }
                }
                _ => VecDeque::new(),
            };
            instructions.push_back(Instruction::Get);
            instructions
        } else {
            VecDeque::from(vec![Instruction::Get])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_new_time() {
        let internal: AcceptorInternal<u32> = AcceptorInternal::new(0);
        let mut acceptor = Acceptor::new(internal);
        let tx = acceptor.in_channel.tx();

        let prop_id = AgentID::Proposer(0);
        let (tx_prop, rx_prop) = channel::unbounded();

        acceptor.out_channels.insert(prop_id, tx_prop);

        tx.send(Message::NewTime(1, prop_id)).unwrap();

        let mut instructions = VecDeque::new();
        acceptor
            .run_command(Instruction::Get, &mut instructions)
            .unwrap();

        assert_eq!(acceptor.internal.time, 1);

        let command = instructions.pop_front().unwrap();

        acceptor.run_command(command, &mut instructions).unwrap();
        let rec = rx_prop.recv().unwrap();

        let expected_msg = Message::UpdatedTime(1, None, None, AgentID::Acceptor(0));
        assert_eq!(rec, expected_msg);

        let command = instructions.pop_front().unwrap();
        assert_eq!(command, Instruction::Get);

        tx.send(Message::NewTime(0, prop_id)).unwrap();
        acceptor.run_command(command, &mut instructions).unwrap();
        assert_eq!(acceptor.internal.time, 1);
    }

    #[test]
    fn test_parse_proposal() {
        let internal = AcceptorInternal::new(0);
        let mut acceptor = Acceptor::new(internal);
        let tx = acceptor.in_channel.tx();

        let (tx_l, rx_l) = channel::unbounded();

        acceptor.out_channels.insert(AgentID::Learner(0), tx_l);
        acceptor.internal.learners.insert(AgentID::Learner(0));

        let prop_id = AgentID::Proposer(0);
        acceptor.internal.proposers.insert(prop_id);

        tx.send(Message::Proposal(1, vec![1, 2, 3], AgentID::Proposer(0)))
            .unwrap();

        let mut instructions = VecDeque::new();
        acceptor
            .run_command(Instruction::Get, &mut instructions)
            .unwrap();

        assert_eq!(acceptor.internal.accepted_time, Some(1));
        assert_eq!(acceptor.internal.accepted_value, Some(vec![1, 2, 3]));

        let command = instructions.pop_front().unwrap();
        acceptor.run_command(command, &mut instructions).unwrap();

        let vote = rx_l.recv().unwrap();
        assert_eq!(
            vote,
            Message::NewVote(acceptor.internal.id, 1, vec![1, 2, 3])
        );
    }
}
