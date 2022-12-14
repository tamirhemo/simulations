use paxos::actors::*;
use paxos::setup_paxos;
use std::time::Duration;
use system::tokio::sync::ActorType;
use system::CrossbeamSystem;

#[tokio::main]
async fn main() {
    let num_of_learners: usize = 30;
    let num_of_acceptors: usize = 20;
    let num_of_proposers: usize = 10;
    let modulus = 17;
    let timeout = Duration::from_secs(1);
    let rng_range = 500;

    let initial_values: Vec<(String, u32, Duration)> = (0..num_of_proposers)
        .map(|i| -> String {
            let k = i % modulus;
            format!("The answer is {}", k)
        })
        .map(|val| (val, rng_range, timeout))
        .collect();

    println!("Building the system...");
    let paxos = setup_paxos(
        CrossbeamSystem::new(),
        initial_values,
        num_of_acceptors,
        num_of_learners,
        ActorType::Light,
    );
    println!("Runnning...");

    let mut verdicts: Vec<String> = paxos
        .run()
        .unwrap()
        .into_values()
        .map(|m| match m {
            Some(Message::Terminated(_, val)) => val,
            _ => String::from("No Yo"),
        })
        .collect();

    assert!(verdicts.windows(2).all(|a| a[0] == a[1]));

    let verdict = verdicts.pop();

    if let Some(s) = verdict {
        println!("A consensus has been reached! {}", s);
    }
}
