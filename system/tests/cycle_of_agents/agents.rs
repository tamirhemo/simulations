// An integration test for the message passing system

use system::{AgentInternal, NextState, Sender};

#[derive(Debug)]
pub struct CycleInternal {
    input_key: Option<usize>,
    output_key: Option<usize>,
    starter: bool,
}

impl CycleInternal {
    pub fn new(starter: bool) -> Self {
        CycleInternal {
            input_key: None,
            output_key: None,
            starter: starter,
        }
    }
}

impl AgentInternal for CycleInternal {
    type Message = usize;
    type Error = ();
    type Key = usize;

    fn new_incoming_key(&mut self, key: &Self::Key) {
        assert!(self.input_key.is_none());
        self.input_key = Some(*key);
    }

    fn new_outgoing_key(&mut self, key: &Self::Key) {
        assert!(self.output_key.is_none());
        self.output_key = Some(*key);
    }

    fn start(&mut self, tx: &mut Sender<Self::Key, Self::Message>) -> Result<NextState<Self::Message>, Self::Error> {
        if self.starter {
            let out = self.output_key.unwrap();
            tx.send(out, 0).unwrap();
        }
        Ok(NextState::Get)
    }

    fn process_message(&mut self, message: Option<Self::Message>, tx: &mut  Sender<Self::Key, Self::Message>) -> Result<NextState<Self::Message>, Self::Error> {
        assert!(message.is_some());
        let value = message.unwrap();

        let out = self.output_key.unwrap();
        tx.send(out, value+1).unwrap();
        Ok(NextState::Terminate(value+1))
    }
}
