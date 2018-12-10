use crate::vpcap::Vpcap;
use crate::simplenet::SimplePacket;

use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    ecs::prelude::{Join, Read, ReadStorage, System, WriteStorage},
    input::InputHandler,
};

pub struct PacketSystem;

impl<'s> System<'s> for PacketSystem {
    type SystemData = (
        ReadStorage<'s, SimplePacket>,
        Read<'s, Time>,
        //Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (packets, time): Self::SystemData) {
        for packet in (&packets).join(){
            println!("{:?}", packet);
        }
    }
}