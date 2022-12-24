use super::actor::*;
use super::channel::{InChannel, OutChannels};
use crate::internal::*;
use crate::System;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::thread;


/// A system of actors  
pub struct SyncSystem<I: ActorInterface> {
    actors: HashMap<I::Key, Actor<I>>,
    terminals: HashSet<I::Key>,
}

/// An error that can occur when running a system.
#[derive(Debug, Clone)]
pub enum SystemError<I, S> {
    ActorError(I),
    ThreadError(S),
}

/// An implementation that assumes keys match with Actor identifiers.
impl<I: ActorInterface> SyncSystem<I> {
    pub fn new() -> Self {
        SyncSystem {
            actors: HashMap::new(),
            terminals: HashSet::new(),
        }
    }

    pub fn run(self) -> Option<HashMap<I::Key, Option<I::Message>>> {
        let mut terminal_handles = HashMap::new();
        for (key, mut actor) in self.actors {
            let handle = thread::spawn(move || actor.run());

            if self.terminals.contains(&key) {
                terminal_handles.insert(key, handle);
            }
        }

        let mut terminal_values = HashMap::new();

        for (k, h) in terminal_handles {
            let value = h.join().unwrap().unwrap();
            terminal_values.insert(k, value);
        }

        Some(terminal_values)
    }
}

pub struct SyncParameters;

impl<T, S, R> From<(T, S, R)> for SyncParameters {
    fn from(_: (T, S, R)) -> Self {
        SyncParameters {}
    }
}

impl<I: ActorInterface> System for SyncSystem<I> {
    type Internal = I::Internal;
    type ActorParameters = SyncParameters;

    fn add_actor(&mut self, key: I::Key, internal: I::Internal, _: Option<SyncParameters>) {
        self.actors.insert(key, Actor::new(internal));
    }

    fn add_channel(&mut self, sender: &I::Key, reciever: &I::Key) {
        let tx = self.actors.get(reciever).unwrap().in_channel.tx();

        self.actors.entry(*sender).and_modify(|s| {
            s.internal.new_outgoing_key(reciever);
            s.out_channels.insert(*reciever, tx);
        });

        self.actors
            .entry(*reciever)
            .and_modify(|a| a.internal.new_incoming_key(sender));
    }

    fn add_terminal(&mut self, key: I::Key) {
        self.terminals.insert(key);
    }
}
