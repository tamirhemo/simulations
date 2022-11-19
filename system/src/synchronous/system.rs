use super::agent::Agent;
use super::channel::{InChannel, OutChannels};
use crate::internal::Internal;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;
use std::thread;

#[derive(Debug, Clone)]
pub struct System<K, A> {
    pub agents: HashMap<K, A>,
    pub terminals: HashSet<K>,
}

#[derive(Debug, Clone)]
pub enum SystemError<A, S> {
    AgentError(A),
    ThreadError(S),
}

/// An implementation that assumes keys match with agent identifiers.
impl<I, K, T, S, R> System<K, Agent<I, S, R>>
where
    I: Internal<Key = K, Message = T>,
    S: OutChannels<Key = K, Message = T>,
    R: InChannel<Sender = S::Sender, Message = T>,
    T: std::fmt::Debug,
    <I as Internal>::Error: std::fmt::Debug,
{
    pub fn new() -> Self {
        System {
            agents: HashMap::new(),
            terminals: HashSet::new(),
        }
    }

    pub fn add_agent(&mut self, key: K, internal: I)
    where
        K: Eq + Hash,
    {
        self.agents.insert(key, Agent::new(internal));
    }

    pub fn add_channel(&mut self, sender: &K, reciever: &K)
    where
        K: Eq + Hash + Copy,
    {
        let tx = self.agents.get(reciever).unwrap().in_channel.tx();

        self.agents.entry(*sender).and_modify(|s| {
            s.internal.new_outgoing_key(reciever);
            s.out_channels.insert(*reciever, tx);
        });

        self.agents
            .entry(*reciever)
            .and_modify(|a| a.internal.new_incoming_key(sender));
    }

    pub fn add_terminal(&mut self, key: K)
    where
        K: Eq + Hash + Copy,
    {
        self.terminals.insert(key);
    }

    pub fn run(self) -> Result<HashMap<K, T>, Box<dyn Error + Send + 'static>>
    where
        I: Send + 'static,
        S: Send + 'static,
        R: Send + 'static,
        T: Send + Debug + 'static,
        <I as Internal>::Error: Send + Debug + 'static,
        K: Eq + Hash + Copy,
    {
        let mut terminal_handles = HashMap::new();
        for (key, mut agent) in self.agents {
            let handle = thread::spawn(move || agent.run());

            if self.terminals.contains(&key) {
                terminal_handles.insert(key, handle);
            }
        }

        let mut terminal_values = HashMap::new();

        for (k, h) in terminal_handles {
            let value = h.join().unwrap().unwrap();
            terminal_values.insert(k, value);
        }

        Ok(terminal_values)
    }
}

