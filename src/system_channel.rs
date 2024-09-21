use crate::{BoxedSystem, SystemFuture};
use async_channel::{Receiver, Sender};
use std::sync::OnceLock;

pub fn drain_system_channel() {
    while let Ok(_) = system_rx().try_recv() {}
}

pub(crate) async fn send_system(system: RunSystem) {
    system_channel()
        .tx
        .send(system)
        .await
        .unwrap_or_else(|_| unreachable!("Channel is static and never closed"))
}
pub(crate) fn system_rx() -> &'static Receiver<RunSystem> {
    &system_channel().rx
}

fn system_channel() -> &'static SystemChannel {
    static SYSTEM_CHANNEL: OnceLock<SystemChannel> = OnceLock::new();

    SYSTEM_CHANNEL.get_or_init(|| {
        let (tx, rx) = async_channel::unbounded();
        SystemChannel { tx, rx }
    })
}

pub(crate) struct SystemChannel {
    pub tx: Sender<RunSystem>,
    pub rx: Receiver<RunSystem>,
}

pub(crate) struct RunSystem {
    pub system: BoxedSystem,
    pub next_future_tx: Sender<SystemFuture>,
}
