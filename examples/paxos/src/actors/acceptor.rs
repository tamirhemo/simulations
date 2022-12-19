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

impl<T: Clone + Eq + Send + Debug + 'static> ActorInternal for AcceptorInternal<T> {
    type Message = Message<T>;
    type Key = AgentID;
    type Error = AgentError<T>;

    fn new_incoming_key(&mut self, _: &Self::Key) {}

    fn new_outgoing_key(&mut self, key: &Self::Key) {
        match key {
            AgentID::Learner(_) => self.learners.insert(*key),
            _ => false,
        };
    }

    fn start<S: Sender<Key = AgentID, Message = Message<T>>>(
        &mut self,
        _tx: &mut S,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        Ok(NextState::Get)
    }

    fn process_message<S: Sender<Key = AgentID, Message = Message<T>>>(
        &mut self,
        message: Option<Message<T>>,
        tx: &mut S,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        if let Some(msg) = message {
            match msg {
                Message::NewTime(ts, id) => {
                    self.proposers.insert(id);
                    if let Some(m) = self.parse_new_time(ts) {
                        tx.send(&id, m)?;
                    }
                }

                Message::Proposal(ts, value, id) => {
                    assert!(self.proposers.contains(&id));
                    if self.parse_proposal(ts, value) {
                        let vote = self.make_vote().unwrap();

                        for &id in self.learners.iter() {
                            tx.send(&id, vote.clone()).ok();
                        }
                    }
                }
                _ => return Err(AgentError::WrongMessageType),
            };
            Ok(NextState::Get)
        } else {
            Ok(NextState::Get)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use system::internal::Instruction;
    #[test]
    fn test_parse_new_time() {
        let mut acceptor: AcceptorInternal<u32> = AcceptorInternal::new(0);

        let prop_id = AgentID::Proposer(0);
        let mut instructions = VecDeque::new();

        let next_state = acceptor
            .process_message(Some(Message::NewTime(1, prop_id)), &mut instructions)
            .unwrap();

        assert_eq!(acceptor.time, 1);
        assert_eq!(next_state, NextState::Get.into());

        let message_sent = match instructions.pop_front().unwrap() {
            Instruction::Send(_, m) => m,
            _ => panic!("falied"),
        };
        assert_eq!(
            message_sent,
            Message::UpdatedTime(1, None, None, AgentID::Acceptor(0))
        );


        let next_state = acceptor
            .process_message(Some(Message::NewTime(0, prop_id)), &mut instructions)
            .unwrap();
        assert_eq!(acceptor.time, 1);
    }

    #[test]
    fn test_parse_proposal() {
        let mut acceptor = AcceptorInternal::new(0);

        let mut instructions = VecDeque::new();

        acceptor.new_outgoing_key(&AgentID::Learner(0));
        acceptor.new_incoming_key(&AgentID::Proposer(0));

        acceptor.proposers.insert(AgentID::Proposer(0));

        acceptor
            .process_message(
                Some(Message::Proposal(1, vec![1, 2, 3], AgentID::Proposer(0))),
                &mut instructions,
            )
            .unwrap();

        assert_eq!(acceptor.accepted_time, Some(1));
        assert_eq!(acceptor.accepted_value, Some(vec![1, 2, 3]));
    }
}


