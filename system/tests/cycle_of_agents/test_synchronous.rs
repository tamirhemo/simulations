use super::setup;
use system::CrossbeamSystem;


#[test]
fn test_sync_cycle() {
    let n = 100;
    let cycle = setup(CrossbeamSystem::new(), n);

    let values = cycle.run().unwrap();
    assert_eq!(values.get(&0), Some(&n));
}
