mod agents;
mod test_synchronous;
mod test_tokio;

use agents::CycleInternal;
use system::tokio::sync::AgentType;
use system::System;

pub fn setup<S: System<Internal = CycleInternal>>(mut system: S, n: usize) -> S
where
    S::AgentParameters: From<(AgentType, usize, usize)>,
{
    system.add_agent(
        0,
        CycleInternal::new(true),
        (AgentType::Light, 2 * n, 2 * n).into(),
    );

    for i in 1..n {
        system.add_agent(
            i,
            CycleInternal::new(false),
            (AgentType::Light, 2 * n, 2 * n).into(),
        );
        system.add_channel(&(i - 1), &i);
    }

    system.add_channel(&(n - 1), &0);
    system.add_terminal(0);

    system
}
