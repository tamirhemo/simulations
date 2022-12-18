mod actors;
mod test_synchronous;
mod test_tokio;

use actors::CycleInternal;
use system::tokio::sync::ActorType;
use system::System;

pub fn setup<S: System<Internal = CycleInternal>>(mut system: S, n: usize) -> S
where
    S::ActorParameters: From<(ActorType, usize, usize)>,
{
    system.add_actor(
        0,
        CycleInternal::new(true),
        Some((ActorType::Light, 2 * n, 2 * n).into()),
    );

    for i in 1..n {
        system.add_actor(
            i,
            CycleInternal::new(false),
            Some((ActorType::Light, 2 * n, 2 * n).into()),
        );
        system.add_channel(&(i - 1), &i);
    }

    system.add_channel(&(n - 1), &0);
    system.add_terminal(0);

    system
}
