use crate::agents::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub mod internal;

#[derive(Clone, Debug)]
pub struct LearnerInternal<T> {
    pub id: AgentID,
    value: Option<T>,
    votes: HashMap<TimeStamp, HashMap<T, HashSet<usize>>>,
    pub num_of_acceptors: usize,
}

impl<T> LearnerInternal<T> {
    pub fn new(id: usize) -> Self {
        LearnerInternal {
            id: AgentID::Learner(id),
            value: None,
            votes: HashMap::new(),
            num_of_acceptors: 0,
        }
    }

    pub fn set_num_of_acceptors(&mut self, num: usize) {
        self.num_of_acceptors = num;
    }

    pub fn value(&self) -> Option<T>
    where
        T: Clone,
    {
        self.value.clone()
    }

    pub fn parse_message(&mut self, msg: Message<T>) -> Result<(), AgentError<T>>
    where
        T: Clone + Hash + Eq,
    {
        let (id, ts, value) = match msg {
            Message::NewVote(AgentID::Acceptor(id), ts, value) => (id, ts, value),
            _ => return Err(AgentError::WrongMessageType),
        };

        self.record_vote(id, ts, value.clone())?;

        if let Some(hash) = self.votes.get(&ts) {
            if let Some(set) = hash.get(&value) {
                if set.len() > (self.num_of_acceptors) / 2 {
                    self.value = Some(value);
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// Record a vote from an acceptor
    pub fn record_vote(&mut self, id: usize, ts: TimeStamp, value: T) -> Result<(), AgentError<T>>
    where
        T: Clone + Hash + Eq,
    {
        self.votes
            .entry(ts)
            .or_insert_with(|| HashMap::new())
            .entry(value)
            .and_modify(|set| {
                set.insert(id);
            })
            .or_insert_with(|| {
                let mut set = HashSet::new();
                set.insert(id);
                set
            });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_message() {
        let mut internal: LearnerInternal<String> = LearnerInternal::new(0);

        internal.set_num_of_acceptors(4);

        let id = { AgentID::Acceptor };
        internal
            .parse_message(Message::NewVote(id(0), 1, String::from("Hello")))
            .unwrap();
        internal
            .parse_message(Message::NewVote(id(1), 1, String::from("Hello")))
            .unwrap();
        internal
            .parse_message(Message::NewVote(id(2), 1, String::from("Hello")))
            .unwrap();

        assert_eq!(internal.value, Some(String::from("Hello")));
    }
}
