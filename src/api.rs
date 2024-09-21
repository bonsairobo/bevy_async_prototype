use crate::{
    system_channel::{send_system, RunSystem},
    BoxedSystem, SystemFuture, SystemFutureOutput, Systems,
};
use bevy::{prelude::*, utils::ConditionalSendFuture};
use futures_util::future::try_join_all;
use futures_util::{StreamExt, TryStreamExt};
use std::future::Future;

/// Run `system` asynchronously.
///
/// `system` may return a future (via [`next_future`]) to be run on the same
/// task (without spawning a new task).
///
/// A Bevy app must be running the [`SystemReceiverPlugin`](crate::SystemReceiverPlugin)
/// for this system to be consumed.
pub async fn run_system<S, M>(system: S) -> anyhow::Result<()>
where
    S: IntoSystem<(), Option<SystemFuture>, M>,
{
    enqueue_boxed_system(box_system(system)).await?.await
}

/// Like `run_system`, but with an iterator of systems to be run in order.
pub async fn run_systems<S>(systems: impl IntoIterator<Item = S>) -> anyhow::Result<()>
where
    S: Into<Systems>,
{
    // Queue systems in order so they are serialized in the expected order.
    let mut all_futures = Vec::new();
    for s in systems {
        match s.into() {
            Systems::None => {}
            Systems::One(system) => all_futures.push(enqueue_boxed_system(system).await?),
            Systems::Many(systems) => {
                for system in systems {
                    all_futures.push(enqueue_boxed_system(system).await?);
                }
            }
        }
    }
    // Since we are guaranteed that the systems have been serialized, now we
    // don't care in what order the futures are polled.
    try_join_all(all_futures).await?;
    Ok(())
}

async fn enqueue_boxed_system(
    system: BoxedSystem,
) -> anyhow::Result<impl Future<Output = anyhow::Result<()>>> {
    let (next_future_tx, next_future_rx) = async_channel::unbounded();
    send_system(RunSystem {
        system,
        next_future_tx,
    })
    .await;
    Ok(async move {
        next_future_rx
            .map(anyhow::Ok)
            .try_for_each_concurrent(None, |next_future| async move {
                let next_systems = next_future.await?;
                run_systems(std::iter::once(next_systems)).await?;
                Ok(())
            })
            .await?;
        Ok(())
    })
}

/// Convenience function to box a system.
///
/// This is normally used for collecting multiple systems into [`Systems`], to
/// be run with [`run_systems`].
pub fn box_system<S, M>(system: S) -> BoxedSystem
where
    S: IntoSystem<(), Option<SystemFuture>, M>,
{
    Box::new(IntoSystem::into_system(system))
}

/// Convenience function to wrap a future to be returned from a system.
pub fn next_future(
    future: impl ConditionalSendFuture<Output = SystemFutureOutput> + 'static,
) -> Option<SystemFuture> {
    Some(Box::pin(future))
}
