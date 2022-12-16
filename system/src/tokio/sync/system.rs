use super::actor::*;
use super::actor_core::{AgentCore, AgentType};
use super::channel::Channels;
use crate::internal::*;
use crate::System;
use std::collections::{HashMap, HashSet};
use tokio;
use tokio::sync::mpsc;

use std::fmt::Debug;
#[derive(Debug)]
pub struct TokioSystem<I: Internal> {
    //pub interfaces: HashMap<I::Key, Interface<I>>,
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

impl<I: Internal> TokioSystem<I> {
    pub fn new(terminals_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(terminals_size);
        TokioSystem {
            agents: HashMap::new(),
            terminals: HashSet::new(),
            tx_term: tx,
            rx_term: rx,
        }
    }

    pub async fn run(mut self) -> Result<Vec<I::Message>, SystemError> {
        // Spawn threads for agents
        for (key, agent) in self.agents {
            let (core, mut interface) = agent.split();
            match core {
                AgentCore::Light(mut core) => {
                    tokio::spawn(async move { core.run().await.ok() });
                }
                AgentCore::Blocking(mut core) => {
                    tokio::task::spawn_blocking(move || {
                        core.run().ok();
                    });
                }
                AgentCore::Heavy(mut core) => {
                    std::thread::spawn(move || core.run().ok());
                }
            }

            let tx = match self.terminals.contains(&key) {
                true => Some(self.tx_term.clone()),
                false => None,
            };

            tokio::spawn(async move { interface.run(tx).await });
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

pub struct Parameters {
    pub kind: AgentType,
    pub buffer: usize,
    pub internal_buffer: usize,
}

impl Parameters {
    pub fn new(kind: AgentType, buffer: usize, internal_buffer: usize) -> Self {
        Parameters {
            kind,
            buffer,
            internal_buffer,
        }
    }
}

impl From<(AgentType, usize, usize)> for Parameters {
    fn from(para_tuple: (AgentType, usize, usize)) -> Self {
        let (kind, buffer, internal_buffer) = para_tuple;

        Parameters {
            kind,
            buffer,
            internal_buffer,
        }
    }
}

impl<I: Internal> System for TokioSystem<I> {
    type Internal = I;
    type ActorParameters = Parameters;

    fn add_terminal(&mut self, key: I::Key) {
        self.terminals.insert(key);
    }

    fn add_actor(&mut self, key: I::Key, internal: I, parameters: Option<Parameters>) {
        let param = parameters.unwrap();
        let (kind, buffer, internal_buffer) = (
            param.kind,
            param.buffer,
            param.internal_buffer,
        );
        let agent = Agent::new(internal, kind, buffer, internal_buffer);

        self.agents.insert(key, agent);
    }

    fn add_channel(&mut self, sender: &I::Key, reciever: &I::Key) {
        let tx = self.agents.get(reciever).unwrap().tx_channel();

        self.agents.entry(*sender).and_modify(|agent| {
            agent.insert_outgoing_channel(*reciever, tx);
        });

        self.agents.entry(*sender).and_modify(|interface| {
            interface.new_outgoing_key(reciever);
        });

        self.agents
            .entry(*reciever)
            .and_modify(|interface| interface.new_incoming_key(sender));
    }
}
