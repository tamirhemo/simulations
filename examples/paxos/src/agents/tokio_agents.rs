use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;
use system::tokio::sync::{AgentType, SyncAgent, SyncSystem};

use super::synchronous::AgentInternal;
use super::*;
use super::{acceptor, learner, proposer};
use acceptor::AcceptorInternal;
use learner::LearnerInternal;
use proposer::ProposerInternal;

/// Paxos implemented in tokio
///
///
pub type PaxosAgent<T> = SyncAgent<AgentInternal<T>, AgentID, Message<T>>;
pub type PaxosSystem<T> = SyncSystem<AgentInternal<T>, AgentID, Message<T>>;

pub fn setup_paxos<T>(
    proposer_initial_values: Vec<(T, TimeStamp, Duration)>,
    n_acceptors: usize,
    n_learners: usize,
    kind: AgentType,
) -> PaxosSystem<T>
where
    T: Clone + Eq + Hash + Debug + Send + 'static,
{
    let mut system: PaxosSystem<T> = PaxosSystem::new(n_learners);
    let buffer = 10000;
    let internal_buffer = 1000;
    let kind = kind;

    // Initialize acceptors
    for i in 0..n_acceptors {
        let internal = AgentInternal::Acceptor(AcceptorInternal::new(i));
        system.add_agent(
            internal.id(),
            internal,
            false,
            kind,
            buffer,
            internal_buffer,
        );
    }

    // Initialize Learners
    // Add Acceptor->Learner channels
    for i in 0..n_learners {
        let internal = AgentInternal::Learner(LearnerInternal::new(i));
        let id = internal.id();
        system.terminals.insert(id);
        system.add_agent(id, internal, true, kind, buffer, internal_buffer);
        for j in 0..n_acceptors {
            system.add_channel(AgentID::Acceptor(j), id);
        }
    }

    // Initialize proposers
    // Add proposer->Acceptor and Acceptor->Proposer channels
    for (i, (val, range, timeout)) in proposer_initial_values.into_iter().enumerate() {
        let internal = AgentInternal::Proposer(ProposerInternal::new(i, val, range, timeout));
        let id = internal.id();
        system.add_agent(id, internal, false, kind, buffer, internal_buffer);
        for j in 0..n_acceptors {
            //if j > n_acceptors/2 {break};
            system.add_channel(AgentID::Acceptor(j), id);
            system.add_channel(id, AgentID::Acceptor(j));
        }
    }
    system
}
