// An integration test for the message passing system

// 

use std::collections::VecDeque;
use system::{Instruction, Internal};

#[derive(Debug)]
pub struct CycleInternal {
    input_key: Option<usize>,
    output_key: Option<usize>,
    starter : bool,
}

impl CycleInternal {
    pub fn new(starter: bool) -> Self {
        CycleInternal { input_key: None, output_key: None, starter: starter }
    }
}

impl Internal for CycleInternal {
    type Message = usize;
    type Error = ();
    type Key = usize;
    type Queue = VecDeque<Instruction<usize, usize>>;

    fn new_incoming_key(&mut self, key: &Self::Key) {
        assert!(self.input_key.is_none());
        self.input_key = Some(*key);
    }

    fn new_outgoing_key(&mut self, key: &Self::Key) {
        assert!(self.output_key.is_none());
        self.output_key = Some(*key);
    }

    fn start(&mut self) -> Self::Queue {
        let mut instructions = VecDeque::new();
        if self.starter {
            let out = self.output_key.unwrap();
            instructions.push_back(Instruction::Send(out, 0));
        }
        instructions.push_back(Instruction::Get);

        instructions
    }

    fn process_message(&mut self, message: Option<Self::Message>) -> Self::Queue {
        assert!(message.is_some());
        let value = message.unwrap();
    
        let out = self.output_key.unwrap();
        VecDeque::from(vec![Instruction::Send(out, value + 1), Instruction::Terminate(value+1)])
    }
}
