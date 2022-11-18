use super::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;
use system::synchronous::crossbeam::AgentCB;
use system::synchronous::system::System;
use system::synchronous::{Agent, Internal};
use system_derive::Internal;

use super::{acceptor, learner, proposer};
use acceptor::AcceptorInternal;
use learner::LearnerInternal;
use proposer::ProposerInternal;

#[derive(Clone, Debug, Internal)]
pub enum AgentInternal<T>
where
    T: Clone + Eq + Hash + Debug,
{
    Learner(LearnerInternal<T>),
    Proposer(ProposerInternal<T>),
    Acceptor(AcceptorInternal<T>),
}

impl<T: Clone + Eq + Hash + Debug> AgentInternal<T> {
    pub fn id(&self) -> AgentID {
        match self {
            AgentInternal::Learner(internal) => internal.id,
            AgentInternal::Proposer(internal) => internal.id,
            AgentInternal::Acceptor(internal) => internal.id,
        }
    }
}

pub type PaxosAgent<T, S, R> = Agent<AgentInternal<T>, S, R>;

/// Paxos implemented using the crossbeam_channel crate
pub type PaxosAgentCB<T> = AgentCB<AgentInternal<T>, AgentID, Message<T>>;
pub type PaxosCB<T> = System<AgentID, PaxosAgentCB<T>>;

pub fn setup_paxos<T>(
    proposer_initial_values: Vec<(T, TimeStamp, Duration)>,
    n_acceptors: usize,
    n_learners: usize,
) -> PaxosCB<T>
where
    T: Clone + Eq + Hash + Debug,
{
    let mut system: PaxosCB<T> = PaxosCB::new();

    // Initialize acceptors
    for i in 0..n_acceptors {
        let internal = AgentInternal::Acceptor(AcceptorInternal::new(i));
        system.add_agent(internal.id(), internal);
    }

    // Initialize Learners
    // Add Acceptor->Learner channels
    for i in 0..n_learners {
        let internal = AgentInternal::Learner(LearnerInternal::new(i));
        let id = internal.id();
        system.terminals.insert(id);
        system.add_agent(id, internal);
        for j in 0..n_acceptors {
            system.add_channel(&AgentID::Acceptor(j), &id);
        }
    }

    // Initialize proposers
    // Add proposer->Acceptor and Acceptor->Proposer channels
    for (i, (val, range, timeout)) in proposer_initial_values.into_iter().enumerate() {
        let internal = AgentInternal::Proposer(ProposerInternal::new(i, val, range, timeout));
        let id = internal.id();
        system.add_agent(id, internal);
        for j in 0..n_acceptors {
            system.add_channel(&AgentID::Acceptor(j), &id);
            system.add_channel(&id, &AgentID::Acceptor(j));
        }
    }
    system
}
