use crate::systems::{HostSystem, PacketSystem};
use amethyst::{
    core::bundle::{Result, SystemBundle},
    ecs::prelude::DispatcherBuilder,
};

/// A bundle is a convenient way to initialise related resources, components and systems in a
/// world. This bundle prepares the world for a game of pong.
pub struct PacketBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for PacketBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(PacketSystem, "packet_system", &[]);
        builder.add(HostSystem, "host_system", &[]);
        Ok(())
    }
}
