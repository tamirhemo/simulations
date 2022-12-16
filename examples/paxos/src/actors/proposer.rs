use crate::actors::*;
use rand::prelude::*;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::Instant;

use super::learner::LearnerInternal;

#[derive(Debug, Clone)]
struct Buffer<T> {
    acc_votes: HashSet<usize>,
    max_time: Option<TimeStamp>,
    value: Option<T>,
    instance: Instant,
}

#[derive(Debug, Clone)]
pub struct ProposerInternal<T> {
    pub id: AgentID,
    time: TimeStamp,
    timeout: Duration,
    rng_range: TimeStamp,
    value: T,
    buffer: Buffer<T>,
    pub acceptors: HashSet<AgentID>,
}

impl<T> Buffer<T> {
    pub fn new() -> Self {
        Buffer {
            acc_votes: HashSet::new(),
            max_time: None,
            value: None,
            instance: Instant::now(),
        }
    }
}

impl<T: Clone> ProposerInternal<T> {
    pub fn new(id: usize, init_value: T, rng_range: TimeStamp, timeout: Duration) -> Self {
        ProposerInternal {
            id: AgentID::Proposer(id),
            time: 0,
            value: init_value,
            timeout,
            rng_range,
            acceptors: HashSet::new(),
            buffer: Buffer::new(),
        }
    }
    pub fn set_new_time(&mut self, time: TimeStamp) -> Result<(), AgentError<T>> {
        if time == self.time {
            return Ok(());
        }

        self.time = time;
        self.buffer = Buffer::new();
        Ok(())
    }

    pub fn parse_updated_time(
        &mut self,
        ts: TimeStamp,
        acc_value: Option<T>,
        acc_t: Option<TimeStamp>,
        acc_id: usize,
    ) -> bool {
        if ts != self.time {
            return false;
        }

        self.buffer.acc_votes.insert(acc_id);

        if acc_t.is_some() && acc_t > self.buffer.max_time {
            self.buffer.value = acc_value;
            self.buffer.max_time = acc_t;
        }

        if self.buffer.acc_votes.len() > self.acceptors.len() / 2 {
            self.update_value();
            return true;
        }
        false
    }

    pub fn update_value(&mut self) {
        if let Some(val) = self.buffer.value.as_ref() {
            self.value = val.clone();
        }
    }

    pub fn parse_message(&mut self, msg: Message<T>) -> Result<bool, AgentError<T>> {
        let (ts, acc_val, acc_t, id) = match msg {
            Message::UpdatedTime(ts, acc_val, acc_t, acc_id) => (ts, acc_val, acc_t, acc_id),
            _ => return Err(AgentError::WrongMessageType),
        };

        let acc_id = match id {
            AgentID::Acceptor(x) => x,
            _ => return Err(AgentError::WrongMessageType),
        };

        let accept = self.parse_updated_time(ts, acc_val, acc_t, acc_id);
        Ok(accept)
    }

    pub fn make_proposal(&self) -> Message<T> {
        Message::Proposal(self.time, self.value.clone(), self.id)
    }
}

impl<T: Clone + Send + Hash + Eq + Debug + 'static> ActorInternal for ProposerInternal<T> {
    type Message = Message<T>;
    type Key = AgentID;
    type Error = AgentInternalError;

    fn new_outgoing_key(&mut self, key: &Self::Key) {
        match key {
            AgentID::Acceptor(_) => self.acceptors.insert(*key),
            _ => panic!("Invalid connection"),
        };
    }

    fn new_incoming_key(&mut self, _: &Self::Key) {}

    fn start(
        &mut self,
        tx: &mut Sender<Self::Key, Self::Message>,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        let mut rng = thread_rng();
        let time: u32 = rng.gen_range(0..self.rng_range);

        new_time(self, time, tx)
    }

    fn process_message(
        &mut self,
        message: Option<Message<T>>,
        tx: &mut Sender<Self::Key, Self::Message>,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        if let Some(msg) = message {
            if self.parse_message(msg).unwrap() {
                let proposal = self.make_proposal();

                for id in self.acceptors.iter() {
                    tx.send(*id, proposal.clone()).unwrap();
                }
            }

            let elapsed = self.buffer.instance.elapsed();
            if self.timeout >= elapsed {
                return Ok(NextState::GetTimeout(self.timeout - elapsed));
            }
        }

        let elapsed = self.buffer.instance.elapsed();
        assert!(elapsed > self.timeout);
        let mut rng = thread_rng();
        let time = self.time + rng.gen_range(0..self.rng_range);
        new_time(self, time, tx)
    }
}

fn new_time<T: Send + 'static + Clone + Eq + Debug + Hash>(
    internal: &mut ProposerInternal<T>,
    time: TimeStamp,
    tx: &mut Sender<AgentID, Message<T>>,
) -> Result<NextState<Message<T>>, <LearnerInternal<T> as ActorInternal>::Error> {
    internal.set_new_time(time).unwrap();
    let message = Message::NewTime(internal.time, internal.id);

    for id in internal.acceptors.iter() {
        tx.send(*id, message.clone()).unwrap();
    }

    Ok(NextState::GetTimeout(internal.timeout))
}

/*


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


*/
