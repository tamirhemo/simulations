# Simulations

This projects aims to implement some known distributed systems algorithms and various methods of testing and debugging them. 

The project is still in very preliminary stages and the code might change substantially. Currently we only have an implementation of the Paxos alogirthm. 

The hope is to make a useful interace to generate simulations and tests automatically, generate testable traces and timelines. 

Comments welcome!

## Paxos
An implementation of the paxos algorithm as agents living on different threads. Each proposer is given a different value 0..N (currently N = 16). The algorithm can be run via
```
$ cargo run --bin paxos_crossbeam
```
