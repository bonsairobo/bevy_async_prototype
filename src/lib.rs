mod api;
mod plugin;
mod system_channel;

pub use api::*;
pub use plugin::*;
pub use system_channel::*;

use bevy::utils::BoxedFuture;

type BoxedSystem = bevy::ecs::system::BoxedSystem<(), Option<SystemFuture>>;

type SystemFuture = BoxedFuture<'static, SystemFutureOutput>;
type SystemFutureOutput = anyhow::Result<Systems>;

pub enum Systems {
    None,
    One(BoxedSystem),
    Many(Vec<BoxedSystem>),
}

impl From<BoxedSystem> for Systems {
    fn from(s: BoxedSystem) -> Self {
        Self::One(s)
    }
}
impl From<Vec<BoxedSystem>> for Systems {
    fn from(s: Vec<BoxedSystem>) -> Self {
        Self::Many(s)
    }
}
