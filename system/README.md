# System

An interace to generate simulations of distributed systems realized as actors passing messages along channels. The user only needs to write an implementation of each Agent's inner logic and a function that sets up the initial conditions. 

Setting up the initial conditions is relatively simple using the system api and does not require any involvement in the inner workings. 

Currently, two possible backends for implementation are available, one based on [`crossbeam_channel`](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/) and one using the asynchronous [`tokio runtime`](https://tokio.rs). The synchronuous version is simpler to run and can be used with any synchronuous message passing interface. The tokio implementation is better for scaling in most cases as it uses threads scheduled by the tokio runtime and does not spawn a standard thread for each agent.

## Example - a cycle of actors 
a system consisting of three actors passing a single message in a circle. One the actors (designated starter) will send the first message and wait to recive it back.

First, we define an actor's internal structure:

```rust
 # #[derive(Debug)]
 pub struct CycleInternal {
    // ID of input actor
    input_key: Option<usize>,
    // ID of output actor
    output_key: Option<usize>,
    // whether the actor is the starter or not.
    starter : bool,
 }
```
To realize CycleInternal as an actor, we need to implement the `ActorInternal` trait.

```rust
impl ActorInternal for CycleInternal {
     type Message = usize;
     type Error = SendError<(usize, usize)>;
     type Key = usize;

     fn new_incoming_key(&mut self, key: &Self::Key) {
         assert!(self.input_key.is_none());
         self.input_key = Some(*key);
     }

     fn new_outgoing_key(&mut self, key: &Self::Key) {
         assert!(self.output_key.is_none());
         self.output_key = Some(*key);
     }

    fn start<S : Sender<Key = Self::Key, Message = Self::Message>>
    (&mut self, tx: &mut S)
     -> Result<NextState<Self::Message>, Self::Error> {
        if self.starter {
            let out = self.output_key.unwrap();
            tx.send(&out, 0).unwrap();
        }
        Ok(NextState::Get)
    }

    fn process_message<S : Sender<Key = Self::Key, Message = Self::Message>>
     (&mut self, message: Option<Self::Message>, tx: &mut  S)
      -> Result<NextState<Self::Message>, Self::Error> {
        assert!(message.is_some());
        let value = message.unwrap();
    
        let out = self.output_key.unwrap();
        // we don't want to panick just because the next actor might be done already
        tx.send(&out, value+1).ok();
        Ok(NextState::Terminate(Some(value+1)))
    }
 }
```

We can now initiate and start the system as follows:

```rust
 use system::synchronous::crossbeam::CrossbeamSystem;
 let mut cycle = CrossbeamSystem::new();

 // Add actors
 cycle.add_actor(0, CycleInternal::new(true), None);
 cycle.add_actor(1, CycleInternal::new(false), None);
 cycle.add_actor(2, CycleInternal::new(false), None);

 // Add channels
 cycle.add_channel(&0, &1);
 cycle.add_channel(&1, &2);
 cycle.add_channel(&2, &0);

 // Make actors 0 terminal
 cycle.add_terminal(0);

 let values = cycle.run().unwrap();
 assert_eq!(values[&0], Some(3));
```



