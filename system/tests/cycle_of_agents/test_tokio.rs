use super::agents::CycleInternal;
use system::tokio::sync;
use system::tokio::sync::AgentType;
use tokio;

pub type Cycle = sync::System<CycleInternal>;

fn setup(n: usize) -> Cycle {
    let mut cycle = Cycle::new(1);

    cycle.add_agent(0, CycleInternal::new(true),  AgentType::Light, 2*n, 2*n);

    for i in 1..n {
        cycle.add_agent(i, CycleInternal::new(false), AgentType::Light, 2*n, 2*n);
        cycle.add_channel(&(i-1), &i);
    }

    cycle.add_channel(&(n-1), &0);
    cycle.add_terminal(0);

    cycle
}

#[test]
fn test_sync_cycle() {
    let n = 1000;
    let cycle = setup(n);

    let threaded_rt = tokio::runtime::Runtime::new().unwrap();

    let values = threaded_rt.block_on(async move { 
        cycle.run().await.unwrap()
    });
    assert_eq!(values.get(0), Some(&n));
}