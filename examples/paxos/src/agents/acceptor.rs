use super::*;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct AcceptorInternal<T> {
    pub id: AgentID,
    pub learners: HashSet<AgentID>,
    pub proposers: HashSet<AgentID>,
    accepted_value: Option<T>,
    accepted_time: Option<TimeStamp>,
    time: TimeStamp,
}

impl<T> AcceptorInternal<T>
where
    T: Clone + Eq,
{
    pub fn new(id: usize) -> Self {
        AcceptorInternal {
            id: AgentID::Acceptor(id),
            learners: HashSet::new(),
            proposers: HashSet::new(),
            accepted_value: None,
            accepted_time: None,
            time: 0,
        }
    }
    pub fn accept_value(&mut self, ts: TimeStamp, value: T) {
        self.accepted_value = Some(value);
        self.accepted_time = Some(ts);
    }

    fn parse_proposal(&mut self, ts: TimeStamp, value: T) -> bool {
        if ts < self.time {
            return false;
        }

        (self.time, self.accepted_value, self.accepted_time) = (ts, Some(value), Some(ts));
        true
    }

    fn parse_new_time(&mut self, ts: TimeStamp) -> Option<Message<T>> {
        if ts <= self.time {
            return None;
        }
        self.time = ts;
        let msg = Message::UpdatedTime(
            self.time,
            self.accepted_value.clone(),
            self.accepted_time,
            self.id,
        );
        Some(msg)
    }

    fn make_vote(&self) -> Result<Message<T>, AgentError<T>> {
        let value = self.accepted_value.clone().unwrap();
        let time = match self.accepted_time {
            Some(ts) => ts,
            _ => return Err(AgentError::NoAcceptedTime),
        };

        Ok(Message::NewVote(self.id, time, value))
    }
}

impl<T: Clone + Eq + Send + Debug + 'static> AgentInternal for AcceptorInternal<T> {
    type Message = Message<T>;
    type Key = AgentID;
    type Error = AgentInternalError;

    fn new_incoming_key(&mut self, _: &Self::Key) {}

    fn new_outgoing_key(&mut self, key: &Self::Key) {
        match key {
            AgentID::Learner(_) => self.learners.insert(*key),
            _ => false,
        };
    }

    fn start(
        &mut self,
        _tx: &mut Sender<Self::Key, Self::Message>,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        Ok(NextState::Get)
    }

    fn process_message(
        &mut self,
        message: Option<Message<T>>,
        tx: &mut Sender<Self::Key, Self::Message>,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        if let Some(msg) = message {
            match msg {
                Message::NewTime(ts, id) => {
                    self.proposers.insert(id);
                    if let Some(m) = self.parse_new_time(ts) {
                        tx.send(id, m).unwrap();
                    }
                }

                Message::Proposal(ts, value, id) => {
                    assert!(self.proposers.contains(&id));
                    if self.parse_proposal(ts, value) {
                        let vote = self.make_vote().unwrap();

                        for &id in self.learners.iter() {
                            tx.send(id, vote.clone()).unwrap();
                        }
                    }
                }
                _ => (),
            };
            Ok(NextState::Get)
        } else {
            Ok(NextState::Get)
        }
    }
}

/*
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

*/
