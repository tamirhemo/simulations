use super::agent::*;
use super::channel::Channels;
use super::interface::{AgentType, Interface};
use super::internal::*;
use std::collections::{HashMap, HashSet};
use tokio;
use tokio::sync::mpsc;

use std::fmt::Debug;
use std::hash::Hash;
#[derive(Debug)]
pub struct System<S, A, K, T> {
    pub interfaces: HashMap<K, S>,
    pub agents: HashMap<K, A>,
    pub terminals: HashSet<K>,
    tx_term: mpsc::Sender<T>,
    rx_term: mpsc::Receiver<T>,
}

#[derive(Debug)]
pub enum SystemError {
    AgentError,
    ThreadError,
}

pub type SyncSystem<I, K, T> = System<
    Interface<I, Option<T>, Instruction<K, T>>,
    Agent<I, Option<T>, Instruction<K, T>, Channels<K, T>>,
    K,
    T,
>;

impl<I, K, T> SyncSystem<I, K, T>
where
    I: SyncInternalQueue<Key = K, Message = T> + Send + Debug + 'static,
    K: Send + Debug + Eq + Hash + Copy + 'static,
    I::Error: Send + Debug + 'static,
    T: Send + Debug + 'static,
    I::Queue: Send + Debug + 'static,
{
    pub fn new(terminals_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(terminals_size);
        System {
            interfaces: HashMap::new(),
            agents: HashMap::new(),
            terminals: HashSet::new(),
            tx_term: tx,
            rx_term: rx,
        }
    }

    pub fn add_agent(
        &mut self,
        key: K,
        internal: I,
        terminal: bool,
        kind: AgentType,
        buffer: usize,
        internal_buffer: usize,
    ) where
        K: Eq + Hash + Copy,
    {
        let (agent, interface) = SyncAgent::new(internal, kind, buffer, internal_buffer);

        self.agents.insert(key, agent);
        self.interfaces.insert(key, interface);

        if terminal {
            self.terminals.insert(key);
        }
    }

    pub fn add_channel(&mut self, sender: K, reciever: K)
    where
        K: Eq + Hash + Copy,
    {
        let tx = self.agents.get(&reciever).unwrap().channels.tx();

        self.agents.entry(sender).and_modify(|agent| {
            agent.channels.insert(reciever, tx);
        });

        self.interfaces.entry(sender).and_modify(|interface| {
            interface.new_outgoing_key(&reciever);
        });

        self.interfaces
            .entry(reciever)
            .and_modify(|interface| interface.new_incoming_key(&sender));
    }

    pub async fn run(mut self) -> Result<Vec<T>, SystemError> {
        let mut interfaces = self.interfaces.into_iter();
        let mut agents = self.agents.into_iter();
        //let mut join_terminals = JoinSet::new();

        // Spawn threads for interfaces
        while let Some((_, interface)) = interfaces.next() {
            match interface {
                Interface::Light(mut core) => {
                    tokio::spawn(async move { core.run().await.ok() });
                }
                Interface::Blocking(mut core) => {
                    tokio::task::spawn_blocking(move || {
                        core.run().ok();
                    });
                }
                Interface::Heavy(mut core) => {
                    std::thread::spawn(move || {
                        core.run().ok()
                    });
                }
            }
        }

        // Spawn threads for agents
        while let Some((key, mut agent)) = agents.next() {
            let tx = match self.terminals.contains(&key) {
                true => Some(self.tx_term.clone()),
                false => None,
            };

            tokio::spawn(async move { agent.run(tx).await });
        }

        let mut terminal_values = Vec::new();
        let terminals_size = self.terminals.len();
        let mut counter = 0;

        while let Some(msg) = self.rx_term.recv().await {
            //println!("got value");
            terminal_values.push(msg);
            counter += 1;
            if counter >= terminals_size {
                break;
            }
        }
        Ok(terminal_values)
    }
}
