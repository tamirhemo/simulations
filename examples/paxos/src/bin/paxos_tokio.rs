use paxos::actors::*;
use paxos::setup_paxos;
use std::time::Duration;
use system::tokio::sync::{ActorType, TokioSystem};

#[tokio::main]
async fn main() {
    let num_of_learners: usize = 10;
    let num_of_acceptors: usize = 100;
    let num_of_proposers: usize = 50;
    let modulus = 17;
    let timeout = Duration::from_secs(10);
    let rng_range = 50;
    let kind = ActorType::Blocking;

    let initial_values: Vec<(String, u32, Duration)> = (0..num_of_proposers)
        .map(|i| -> String {
            let k = i % modulus;
            format!("The answer is {}", k)
        })
        .map(|val| (val, rng_range, timeout))
        .collect();

    println!("Building the system...");
    let paxos = setup_paxos(
        TokioSystem::new(num_of_learners),
        initial_values,
        num_of_acceptors,
        num_of_learners,
        kind,
    );

    println!("Runnning...");
    let verdict_messages = paxos.run().await.unwrap();

    let mut verdicts: Vec<String> = verdict_messages
        .into_iter()
        .map(|m| match m {
            Message::Terminated(_, val) => val,
            _ => String::from("No Yo"),
        })
        .collect();

    assert!(verdicts.windows(2).all(|a| a[0] == a[1]));

    let verdict = verdicts.pop();

    if let Some(s) = verdict {
        println!("A consensus has been reached! {}", s);
    }
}
