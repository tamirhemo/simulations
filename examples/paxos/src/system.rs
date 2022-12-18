use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;
use system::tokio::sync::{ActorType, TokioSystem};
use system::System;

use crate::actors::acceptor::AcceptorInternal;
use crate::actors::learner::LearnerInternal;
use crate::actors::proposer::ProposerInternal;
use crate::actors::*;

/// Setting up Paxos
pub fn setup_paxos<S, T>(
    mut system: S,
    proposer_initial_values: Vec<(T, TimeStamp, Duration)>,
    n_acceptors: usize,
    n_learners: usize,
    kind: ActorType,
) -> S
where
    T: Clone + Eq + Hash + Debug + Send + 'static,
    S: System<Internal = PaxosInternal<T>>,
    S::ActorParameters: From<(ActorType, usize, usize)>,
{
    let buffer = 10000;
    let internal_buffer = 1000;
    let kind = kind;

    // Initialize acceptors
    for i in 0..n_acceptors {
        let internal = PaxosInternal::Acceptor(AcceptorInternal::new(i));
        system.add_actor(
            internal.id(),
            internal,
            Some((kind, buffer, internal_buffer).into()),
        );
    }

    // Initialize Learners
    // Add Acceptor->Learner channels
    for i in 0..n_learners {
        let internal = PaxosInternal::Learner(LearnerInternal::new(i));
        let id = internal.id();
        system.add_terminal(id);
        system.add_actor(id, internal, Some((kind, buffer, internal_buffer).into()));
        system.add_terminal(id);
        for j in 0..n_acceptors {
            system.add_channel(&AgentID::Acceptor(j), &id);
        }
    }

    // Initialize proposers
    // Add proposer->Acceptor and Acceptor->Proposer channels
    for (i, (val, range, timeout)) in proposer_initial_values.into_iter().enumerate() {
        let internal = PaxosInternal::Proposer(ProposerInternal::new(i, val, range, timeout));
        let id = internal.id();
        system.add_actor(id, internal, Some((kind, buffer, internal_buffer).into()));
        for j in 0..n_acceptors {
            //if j > n_acceptors/2 {break};
            system.add_channel(&AgentID::Acceptor(j), &id);
            system.add_channel(&id, &AgentID::Acceptor(j));
        }
    }
    system
}
