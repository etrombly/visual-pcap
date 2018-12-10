use crate::simplenet::SimplePacket;
use crate::vpcap::Vpcap;

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
        for packet in (&packets).join() {
            println!("{:?}", packet);
            /*
            let p: Vec<&SimplePacket> = packets
                .iter()
                .filter_map(|p| match p {
                    Some(x) => Some(x),
                    _ => None,
                })
                .collect();
            println!("{:#?}", p);
            let mut p: Vec<&IpAddr> = packets
                .iter()
                .filter_map(|p| match p {
                    Some(SimplePacket::Ip(x)) => Some(&x.sender_proto_addr),
                    _ => None,
                })
                .collect();
            p.sort();
            p.dedup();
            println!("{:#?}", p);
            */
        }
    }
}
