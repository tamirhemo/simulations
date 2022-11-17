use crate::agents::*;
use std::collections::HashSet;

pub mod mpsc;
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
