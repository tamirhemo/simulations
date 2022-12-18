use super::actors::CycleInternal;
use super::setup;
use system::tokio::sync;
use tokio;

pub type Cycle = sync::TokioSystem<CycleInternal>;

#[test]
fn test_tokio_cycle() {
    let n = 1000;
    let cycle = setup(Cycle::new(n + 1), n);

    let threaded_rt = tokio::runtime::Runtime::new().unwrap();

    let values = threaded_rt.block_on(async move { cycle.run().await.unwrap() });
    assert_eq!(values[0], n);
}
