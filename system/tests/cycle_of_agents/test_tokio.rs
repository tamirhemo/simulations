use super::agents::CycleInternal;
use system::tokio::sync;
use system::tokio::sync::AgentType;
use tokio;
use super::setup;

pub type Cycle = sync::TokioSystem<CycleInternal>;


#[test]
fn test_sync_cycle() {
    let n = 1000;
    let cycle = setup(Cycle::new(n+1), n);

    let threaded_rt = tokio::runtime::Runtime::new().unwrap();

    let values = threaded_rt.block_on(async move { cycle.run().await.unwrap() });
    assert_eq!(values.get(0), Some(&n));
}
