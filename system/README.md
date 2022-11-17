# System

An interace to generate simulations of distributed systems realized as a set of agents and a set of message-passing channels. The user only needs to write an implementation of each Agent's inner logic and a function that sets up the initial conditions. 

Setting up the initial conditions is relatively simple using the system api and does not require any involvement in the inner workings. 

Currently, two possible backends for implementation are available, one based on [`crossbeam_channel`](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/) and one using the asynchronous [`tokio runtime`](https://tokio.rs). 

See [`paxos`](../examples/paxos/) for an example.


