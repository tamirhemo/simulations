use super::agent::*;
use super::channel::Channels;
use super::interface::{AgentType, Interface};
use crate::internal::*;
use std::collections::{HashMap, HashSet};
use tokio;
use tokio::sync::mpsc;

use std::fmt::Debug;
#[derive(Debug)]
pub struct System<I: Internal> {
    pub interfaces: HashMap<I::Key, Interface<I>>,
    pub agents: HashMap<I::Key, Agent<I, Channels<I::Key, I::Message>>>,
    pub terminals: HashSet<I::Key>,
    tx_term: mpsc::Sender<I::Message>,
    rx_term: mpsc::Receiver<I::Message>,
}

#[derive(Debug)]
pub enum SystemError {
    AgentError,
    ThreadError,
}

impl<I: Internal> System<I> {
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
        key: I::Key,
        internal: I,
        kind: AgentType,
        buffer: usize,
        internal_buffer: usize,
    ) {
        let (agent, interface) = Agent::new(internal, kind, buffer, internal_buffer);

        self.agents.insert(key, agent);
        self.interfaces.insert(key, interface);
    }

    pub fn add_channel(&mut self, sender: &I::Key, reciever: &I::Key) {
        let tx = self.agents.get(reciever).unwrap().channels.tx();

        self.agents.entry(*sender).and_modify(|agent| {
            agent.channels.insert(*reciever, tx);
        });

        self.interfaces.entry(*sender).and_modify(|interface| {
            interface.new_outgoing_key(reciever);
        });

        self.interfaces
            .entry(*reciever)
            .and_modify(|interface| interface.new_incoming_key(sender));
    }

    pub fn add_terminal(&mut self, key: I::Key) {
        self.terminals.insert(key);
    }

    pub async fn run(mut self) -> Result<Vec<I::Message>, SystemError> {
        // Spawn threads for interfaces
        for (_, interface) in self.interfaces {
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
                    std::thread::spawn(move || core.run().ok());
                }
            }
        }

        // Spawn threads for agents
        for (key, mut agent) in self.agents {
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
            terminal_values.push(msg);
            counter += 1;
            if counter >= terminals_size {
                break;
            }
        }
        Ok(terminal_values)
    }
}
