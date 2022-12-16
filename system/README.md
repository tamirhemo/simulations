# System

An interace to generate simulations of distributed systems realized as actors passing messages along channels. The user only needs to write an implementation of each Agent's inner logic and a function that sets up the initial conditions. 

Setting up the initial conditions is relatively simple using the system api and does not require any involvement in the inner workings. 

Currently, two possible backends for implementation are available, one based on [`crossbeam_channel`](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/) and one using the asynchronous [`tokio runtime`](https://tokio.rs). The synchronuous version is simpler to run and can be used with any synchronuous message passing interface. The tokio implementation is better for scaling in most cases as it uses threads scheduled by the tokio runtime and does not spawn a standard thread for each agent.

See [`paxos`](../examples/paxos/) for an example.


