use crate::simplenet::*;
use crate::IpAddrS;
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use std::net::IpAddr;

use amethyst::{ecs::prelude::{World, Join}, prelude::*};

pub struct Vpcap;

impl SimpleState for Vpcap {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        world.register::<SimplePacket>();
        world.register::<IpAddrS>();
        load_pcap(world);
    }
}

fn load_pcap(world: &mut World) {
    let mut cap = Capture::from_file("/home/eric/Downloads/test.pcap").expect("error opening pcap");
    let mut hosts = Vec::new();

    while let Ok(p) = cap.next() {
        if let Some(ether_packet) = EthernetPacket::new(&p) {
            if let Some(pack) = handle_ethernet_frame(&ether_packet, &p.header.ts) {
                hosts.push(pack.get_source_ip_addr());
                world.create_entity().with(pack).build();
            }
        }
    }
    hosts.sort();
    hosts.dedup();
    for host in &hosts {
        world.create_entity().with(IpAddrS(*host)).build();
    }
}
