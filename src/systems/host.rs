use crate::IpAddrS;
use crate::vpcap::Vpcap;

use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    ecs::prelude::{Join, Read, ReadStorage, System, WriteStorage},
    input::InputHandler,
};

pub struct HostSystem;

impl<'s> System<'s> for HostSystem {
    type SystemData = (
        ReadStorage<'s, IpAddrS>,
        Read<'s, Time>,
        //Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (hosts, time): Self::SystemData) {
        for host in (&hosts).join() {
            //println!("{:?}", host);
        }
    }
}
