use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, IoTaskPool},
};
use bevy_send_system::*;
use futures_util::future::join;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SystemReceiverPlugin::default())
        .add_systems(Startup, build_hierarchy_async)
        .set_runner(my_runner)
        .run();
}

fn my_runner(mut app: App) -> AppExit {
    loop {
        // println!("In main loop");
        app.update();
        if let Some(exit) = app.should_exit() {
            return exit;
        }
    }
}

// Bevy already has good support for spawning tasks from the ECS. If you think
// about it, spawning a task is essentially just *sending an async computation
// to another thread*. What Bevy lacks is *sending a synchronous ECS system
// back*.
//
// This crate lets you run an ECS system from async code and await the result.
fn build_hierarchy_async() {
    let task = IoTaskPool::get().spawn(async {
        // ... do async stuff ...

        run_system(|mut commands: Commands, time: Res<Time>| {
            let now = time.elapsed().as_secs_f64();
            println!("spawning parent at {now}");
            let parent = commands.spawn(Name::new("parent")).id();

            let child_a = async move {
                // ... do async stuff ...

                box_system(move |mut commands: Commands, time: Res<Time>| {
                    let now = time.elapsed().as_secs_f64();
                    println!("spawning child A at {now}");
                    commands.entity(parent).with_children(|builder| {
                        builder.spawn(Name::new("child A"));
                    });
                    None
                })
            };

            // Let's say you want to spawn on a specific pool here.
            let child_b = AsyncComputeTaskPool::get().spawn(async move {
                // ... do async stuff ...

                box_system(move |mut commands: Commands, time: Res<Time>| {
                    let now = time.elapsed().as_secs_f64();
                    println!("spawning child B at {now}");
                    commands.entity(parent).with_children(|builder| {
                        builder.spawn(Name::new("child B"));
                    });
                    None
                })
            });

            // Join two futures and run their systems sequentially.
            next_future(async move {
                let (sys_a, sys_b) = join(child_a, child_b).await;
                run_systems([sys_a, sys_b]).await?;
                Ok(Systems::None)
            })
        })
        .await
    });

    task.detach();

    // Cancelling the task works as you would expect.
    //
    // Systems are not cancellable.
    // Descendant futures are cancellable.
    // Descendant tasks are independently cancellable.
    //
    // task.cancel();
}
