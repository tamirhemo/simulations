use super::setup;
use system::CrossbeamSystem;

#[test]
fn test_sync_cycle() {
    let n = 80;
    let cycle = setup(CrossbeamSystem::new(), n);

    let values = cycle.run().unwrap();
    assert_eq!(values[&0], Some(n));
}
