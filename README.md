# Simulations

This projects aims to implement some known distributed systems algorithms and various methods of testing and debugging them. 

Comments welcome!

## Systems
[`systems`](./systems/) - a libray for writing simulations of distributed systems.

## Examples
Currently, we have an implementation of the [paxos algorithm](https://en.wikipedia.org/wiki/Paxos_(computer_science)). Each proposer is given a different value 0..N (currently N = 16). The algorithm can be run via
```
$ cargo run --bin paxos_crossbeam
```
Or, for an implementation using the tokio runtime
```
$ cargo run --bin paxos_tokio
```
