# Paxos
An implementation of the [Paxos algorithm](https://en.wikipedia.org/wiki/Paxos_(computer_science)) written using the [`system`](../../system/) library. Each proposer is given a different value 0..N (currently N = 16). The algorithm can be run via

```
$ cargo run --bin paxos_crossbeam

Building the system...
Runnning...
A consensus has been reached! The answer is 1
```

And the output looks like
