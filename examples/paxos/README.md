# Paxos
An implementation of the paxos algorithm as agents living on different threads. Each proposer is given a different value 0..N (currently N = 16). The algorithm can be run via
```
cargo run --bin paxos-crossbeam
```
