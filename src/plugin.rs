use crate::system_channel::{system_rx, RunSystem};
use bevy::{ecs::system::BoxedSystem, prelude::*};
use std::num::NonZeroUsize;

/// Runs systems sent via [`run_system`](crate::run_system) and
/// [`run_systems`](crate::run_systems).
///
/// All systems are run in FIFO order with `&mut World` access.
#[derive(Clone, Debug)]
pub struct SystemReceiverPlugin {
    pub max_systems_per_frame: NonZeroUsize,
}

impl Default for SystemReceiverPlugin {
    fn default() -> Self {
        Self {
            max_systems_per_frame: 32.try_into().unwrap(),
        }
    }
}

impl Plugin for SystemReceiverPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FrameLimit {
            max_systems_per_frame: self.max_systems_per_frame,
        })
        .add_systems(Update, receive_and_run);
    }
}

#[derive(Resource)]
struct FrameLimit {
    max_systems_per_frame: NonZeroUsize,
}

fn receive_and_run(world: &mut World) {
    world.resource_scope(|world, limit: Mut<FrameLimit>| {
        let rx = system_rx();
        for _ in 0..limit.max_systems_per_frame.get() {
            let Ok(RunSystem {
                system,
                next_future_tx,
            }) = rx.try_recv()
            else {
                return;
            };
            println!("Got system");
            if let Some(next_future) = run_boxed_system(world, system) {
                // Failure to send could mean the receiving future was cancelled.
                println!("Sending future back");
                let _ = next_future_tx.send_blocking(next_future);
            }
        }
    });
}

fn run_boxed_system<O: 'static>(world: &mut World, system: BoxedSystem<(), O>) -> O {
    let system_id = world.register_boxed_system(system);
    let out = world.run_system(system_id).unwrap();
    world.remove_system(system_id).unwrap();
    out
}
