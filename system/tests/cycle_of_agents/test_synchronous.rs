use super::agents::CycleInternal;
use system::synchronous::crossbeam;

pub type Cycle = crossbeam::System<CycleInternal>;

fn setup(n: usize) -> Cycle {
    let mut cycle = crossbeam::System::new();

    cycle.add_agent(0, CycleInternal::new(true));

    for i in 1..n {
        cycle.add_agent(i, CycleInternal::new(false));
        cycle.add_channel(&(i - 1), &i);
    }

    cycle.add_channel(&(n - 1), &0);
    cycle.add_terminal(0);

    cycle
}

#[test]
fn test_sync_cycle() {
    let n = 100;
    let cycle = setup(n);

    let values = cycle.run().unwrap();
    assert_eq!(values.get(&0), Some(&n));
}
