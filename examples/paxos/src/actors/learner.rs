use crate::actors::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

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

impl<T> ActorInternal for LearnerInternal<T>
where
    T: Clone + Send + Eq + Hash + Debug + 'static,
{
    type Message = Message<T>;
    type Key = AgentID;
    type Error = AgentInternalError;

    fn new_incoming_key(&mut self, key: &Self::Key) {
        match key {
            AgentID::Acceptor(_) => self.num_of_acceptors += 1,
            _ => panic!("Invalid connection!"),
        }
    }

    fn new_outgoing_key(&mut self, _: &Self::Key) {}

    fn start<S: Sender<Key = AgentID, Message = Message<T>>>(
        &mut self,
        tx: &mut S,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        Ok(NextState::Get)
    }

    fn process_message<S: Sender<Key = AgentID, Message = Message<T>>>(
        &mut self,
        message: Option<Message<T>>,
        tx: &mut S,
    ) -> Result<NextState<Self::Message>, Self::Error> {
        if let Some(msg) = message {
            self.parse_message(msg).unwrap();
        }
        if let Some(val) = &self.value {
            return Ok(NextState::Terminate(Some(Message::Terminated(
                self.id,
                val.clone(),
            ))));
        }
        Ok(NextState::Get)
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

/*

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

*/
