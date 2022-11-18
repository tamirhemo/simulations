use crate::agents::proposer::ProposerInternal;
use crate::agents::*;
use rand::prelude::*;
use std::{collections::VecDeque, fmt::Debug};
use system::internal::{Instruction, Internal};
use system::synchronous::{AgentCB, InChannel, OutChannels};

type Queue<T> = VecDeque<Instruction<AgentID, Message<T>>>;

pub type Proposer<T> = AgentCB<ProposerInternal<T>, AgentID, Message<T>>;

fn new_time<T: Clone + Eq + Debug>(
    internal: &mut ProposerInternal<T>,
    time: TimeStamp,
) -> VecDeque<Instruction<AgentID, Message<T>>> {
    internal.set_new_time(time).unwrap();
    let message = Message::NewTime(internal.time, internal.id);

    let mut instructions: VecDeque<Instruction<AgentID, Message<T>>> = internal
        .acceptors
        .iter()
        .map(|id| Instruction::Send(*id, message.clone()))
        .collect();
    instructions.push_back(Instruction::GetTimeout(internal.timeout));

    instructions
}

impl<T: Clone + Eq + Debug> Internal for ProposerInternal<T> {
    type Message = Message<T>;
    type Key = AgentID;
    type Queue = Queue<T>;
    type Error = AgentInternalError;

    fn new_outgoing_key(&mut self, key: &Self::Key) {
        match key {
            AgentID::Acceptor(_) => self.acceptors.insert(key.clone()),
            _ => panic!("Invalid connection"),
        };
    }

    fn new_incoming_key(&mut self, _: &Self::Key) {
        ()
    }

    fn start(&mut self) -> Self::Queue {
        let mut rng = thread_rng();
        let time: u32 = rng.gen_range(0..self.rng_range);

        new_time(self, time)
    }

    fn process_message(
        &mut self,
        message: Option<Self::Message>,
    ) -> VecDeque<Instruction<Self::Key, Self::Message>> {
        if let Some(msg) = message {
            let mut instructions = VecDeque::new();

            if self.parse_message(msg).unwrap() {
                let proposal = self.make_proposal();

                let mut send_proposal_instructions: VecDeque<
                    Instruction<Self::Key, Self::Message>,
                > = self
                    .acceptors
                    .iter()
                    .map(|id| Instruction::Send(*id, proposal.clone()))
                    .collect();

                instructions.append(&mut send_proposal_instructions);
            }

            let elapsed = self.buffer.instance.elapsed();
            if self.timeout >= elapsed {
                instructions.push_back(Instruction::GetTimeout(self.timeout - elapsed));
                return instructions;
            }
        }

        let elapsed = self.buffer.instance.elapsed();
        assert!(elapsed > self.timeout);
        let mut rng = thread_rng();
        let time = self.time + rng.gen_range(0..self.rng_range);
        new_time(self, time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_make_new_time() {}

    #[test]
    fn test_parse_updated_time() {
        let internal = ProposerInternal::new(0, 5, 10, Duration::from_secs(1));

        let mut proposer = Proposer::new(internal);
        let tx = proposer.in_channel.tx();

        let (tx_acc, _) = channel::unbounded();
        let acc_id = AgentID::Acceptor(0);

        proposer.internal.acceptors.insert(acc_id);
        proposer.out_channels.insert(acc_id, tx_acc);

        tx.send(Message::UpdatedTime(0, None, None, acc_id))
            .unwrap();

        let mut instructions = VecDeque::new();
        proposer
            .run_command(Instruction::Get, &mut instructions)
            .unwrap();

        assert_eq!(proposer.internal.buffer.acc_votes.len(), 1);
        //assert_eq!(proposer.internal.num_of_acceptors, 1);
    }
}
