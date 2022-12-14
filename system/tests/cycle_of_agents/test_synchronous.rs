use super::agents::CycleInternal;
use super::setup;
use system::CrossbeamSystem;
use system::System;

pub type Cycle = CrossbeamSystem<CycleInternal>;

#[test]
fn test_sync_cycle() {
    let n = 100;
    let cycle = setup(CrossbeamSystem::new(), n);

    let values = cycle.run().unwrap();
    assert_eq!(values.get(&0), Some(&n));
}
