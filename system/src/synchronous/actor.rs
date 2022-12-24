use super::channel::{InChannel, OutChannels};
use crate::internal::*;
use std::fmt::Debug;
use std::hash::Hash;


/// An interface for the Actor type.
/// 
/// This trait is used to define the methods that an actor must implement
/// and various compatibilities that must be met, which are enforced by the trait bounds.
pub trait ActorInterface: Send + 'static {

    /// The type of message that the actor will send and recieve.
    type Message: Send + Clone + Debug + 'static;
    /// The type of key that the actor will use to identify other actors.
    type Key: Hash + Send + Copy + Debug + Eq + PartialEq;
    /// The type of error that the actor may return.
    type Error: Send + From<<Self::Internal as ActorInternal>::Error> + Debug;
    /// The type of sender that the actor will use to send messages.
    type Sender;
    /// The type of channel that the actor will use to recieve messages.
    type InChannel: InChannel<Message = Self::Message, Sender = Self::Sender>;
    /// The type of channels that the actor will use to send messages to other actors.
    type OutChannels: OutChannels<Key = Self::Key, Message = Self::Message, Sender = Self::Sender>;
    /// The type defining the internal operations that the actor will perform.
    type Internal: ActorInternal<Key = Self::Key, Message = Self::Message>;
}

/// A container for an Actor.
/// 
/// The actors methods will usually be called by the system and not by the user. 
#[derive(Debug, Clone)]
pub struct Actor<I: ActorInterface> {
    pub internal: I::Internal,
    pub in_channel: I::InChannel,
    pub out_channels: I::OutChannels,
}

impl<I: ActorInterface> Actor<I> {
    pub fn new(internal: I::Internal) -> Self {
        Actor {
            internal,
            in_channel: I::InChannel::new(),
            out_channels: I::OutChannels::new(),
        }
    }

    /// Act with respect to a given next state 
    fn act_next(
        &mut self,
        next_state: NextState<I::Message>,
    ) -> Result<NextState<I::Message>, I::Error> {
        match next_state {
            NextState::Get => {
                let message = self.in_channel.recv();
                return Ok(self
                    .internal
                    .process_message(message, &mut self.out_channels)?);
            }
            NextState::GetTimeout(t) => {
                let message = self.in_channel.recv_timeout(t);
                return Ok(self
                    .internal
                    .process_message(message, &mut self.out_channels)?);
            }
            NextState::Terminate(m) => return Ok(NextState::Terminate(m)),
        }
    }

    pub fn run(&mut self) -> Result<Option<I::Message>, I::Error> {
        let mut next_state = self.internal.start(&mut self.out_channels)?;

        loop {
            if let NextState::Terminate(m) = next_state {
                return Ok(m);
            }
            next_state = self.act_next(next_state)?;
        }
    }
}
