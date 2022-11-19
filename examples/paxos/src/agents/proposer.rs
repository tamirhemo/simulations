use crate::agents::*;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::Instant;

pub mod internal;
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
