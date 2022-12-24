use super::channel::{InChannel, OutChannels};
use crate::internal::*;
use std::fmt::Debug;
use std::hash::Hash;


pub trait ActorInterface: Send + 'static {
    type Message: Send + Clone + Debug + 'static;
    type Key: Hash + Send + Copy + Debug + Eq + PartialEq;
    type Error: Send + From<<Self::Internal as ActorInternal>::Error> + Debug;
    type Sender;

    type InChannel: InChannel<Message = Self::Message, Sender = Self::Sender>;
    type OutChannels: OutChannels<Key = Self::Key, Message = Self::Message, Sender = Self::Sender>;
    type Internal: ActorInternal<Key = Self::Key, Message = Self::Message>;
}

/// A container for an Actor.
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
