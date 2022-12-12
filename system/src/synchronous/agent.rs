use super::channel::{ChannelError, InChannel, OutChannels};
use crate::internal::*;
use std::time::Duration;

/// A container for an agent.
#[derive(Debug, Clone)]
pub struct Agent<I, S, R> {
    pub internal: I,
    pub in_channel: R,
    pub out_channels: S,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentError<N, T> {
    InternalError(N),
    ChannelError(ChannelError<T>),
    ExitedWithoutValue,
}

impl<I: Internal, S, R> Agent<I, S, R>
where
    S: OutChannels<Key = I::Key, Message = I::Message>,
    R: InChannel<Sender = S::Sender, Message = I::Message>,
{
    pub fn new(internal: I) -> Self {
        Agent {
            internal,
            in_channel: <R as InChannel>::new(),
            out_channels: OutChannels::new(),
        }
    }

    fn get(&mut self, instructions: &mut I::Queue, timeout: Option<Duration>) {
        let msg = match timeout {
            None => self.in_channel.recv().ok(),
            Some(t) => self.in_channel.recv_timeout(t).ok(),
        };
        let mut current_instructions = self.internal.process_message(msg);

        instructions.append(&mut current_instructions);
    }

    pub fn run_command(
        &mut self,
        command: Instruction<I::Key, I::Message>,
        instructions: &mut I::Queue,
    ) -> Result<Option<I::Message>, AgentError<I::Error, I::Message>> {
        match command {
            Instruction::Send(k, m) => {
                self.out_channels.send(k, m).ok();
            }

            Instruction::Get => self.get(instructions, None),

            Instruction::GetTimeout(t) => self.get(instructions, Some(t)),

            Instruction::Terminate(val) => return Ok(Some(val)),
        };

        Ok(None)
    }

    pub fn run(&mut self) -> Result<I::Message, AgentError<I::Error, I::Message>> {
        let mut instructions = self.internal.start();
        //assert!(!instructions.is_empty());

        while let Some(command) = instructions.pop_front() {
            let return_value = self.run_command(command, &mut instructions)?;
            if let Some(val) = return_value {
                return Ok(val);
            }
        }
        Err(AgentError::ExitedWithoutValue)
    }
}
