use crossbeam_channel as channel;
use std::fmt::Debug;
use std::hash::Hash;
use system::{ActorInternal, NextState, Sender};
use system_derive::ActorInternal;

pub mod acceptor;
pub mod learner;
pub mod proposer;
//pub mod tokio_agents;

pub type TimeStamp = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Message<T> {
    NewTime(TimeStamp, AgentID),
    Proposal(TimeStamp, T, AgentID),
    Accept(TimeStamp),
    NewVote(AgentID, TimeStamp, T),
    UpdatedTime(TimeStamp, Option<T>, Option<TimeStamp>, AgentID),
    Terminated(AgentID, T),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentID {
    Proposer(usize),
    Acceptor(usize),
    Learner(usize),
}

impl AgentID {
    pub fn is_proposer(&self) -> bool {
        matches!(self, AgentID::Proposer(_))
    }
    pub fn is_leanrer(&self) -> bool {
        matches!(self, AgentID::Learner(_))
    }
    pub fn is_acceptor(&self) -> bool {
        matches!(self, AgentID::Acceptor(_))
    }
}

// Errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentError<T> {
    RecvError(channel::RecvError),
    SendError(channel::SendError<Message<T>>),
    WrongMessageType,
    NoConsensus,
    NoMessage,
    TryRecvError,
    NoAcceptedTime,
    RecvTimeoutError(channel::RecvTimeoutError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentInternalError {
    WrongMessageType,
    NoMessage,
    NoAcceptedTime,
    NoConsensus,
}

/// A Paxos Agnet
#[derive(Debug, ActorInternal)]
pub enum PaxosInternal<T>
where
    T: Send + Clone + 'static + Eq + Hash + PartialEq + Debug,
{
    Learner(learner::LearnerInternal<T>),
    Proposer(proposer::ProposerInternal<T>),
    Acceptor(acceptor::AcceptorInternal<T>),
}

impl<T> PaxosInternal<T>
where
    T: Send + Clone + 'static + Eq + Hash + PartialEq + Debug,
{
    pub fn id(&self) -> AgentID {
        match self {
            PaxosInternal::Learner(internal) => internal.id,
            PaxosInternal::Proposer(internal) => internal.id,
            PaxosInternal::Acceptor(internal) => internal.id,
        }
    }
}

impl<T> From<channel::TryRecvError> for AgentError<T> {
    fn from(_: channel::TryRecvError) -> Self {
        AgentError::TryRecvError
    }
}

impl<T> From<channel::RecvError> for AgentError<T> {
    fn from(err: channel::RecvError) -> Self {
        AgentError::RecvError(err)
    }
}

impl<T> From<channel::SendError<Message<T>>> for AgentError<T> {
    fn from(err: channel::SendError<Message<T>>) -> Self {
        AgentError::SendError(err)
    }
}

impl<T> From<channel::RecvTimeoutError> for AgentError<T> {
    fn from(err: channel::RecvTimeoutError) -> Self {
        AgentError::RecvTimeoutError(err)
    }
}
