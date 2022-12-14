use super::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;
use system::internal::Internal;
use system::synchronous::crossbeam::AgentCB;
use system::synchronous::Agent;
use system::CrossbeamSystem;
use system_derive::Internal;

use super::{acceptor, learner, proposer};
use acceptor::AcceptorInternal;
use learner::LearnerInternal;
use proposer::ProposerInternal;

#[derive(Clone, Debug, Internal)]
pub enum AgentInternal<T>
where
    T: 'static + Send + Clone + Eq + Hash + Debug,
{
    Learner(LearnerInternal<T>),
    Proposer(ProposerInternal<T>),
    Acceptor(AcceptorInternal<T>),
}

impl<T: Clone + Eq + Hash + Debug + Send> AgentInternal<T> {
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
pub type PaxosCB<T> = CrossbeamSystem<PaxosAgentCB<T>>;
