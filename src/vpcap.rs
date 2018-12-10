use crate::simplenet::*;
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use std::net::IpAddr;

use amethyst::{
    assets::{AssetStorage, Loader},
    core::transform::Transform,
    ecs::prelude::World,
    prelude::*,
    renderer::{
        Camera, Flipped, PngFormat, Projection, SpriteRender, SpriteSheet, SpriteSheetFormat,
        SpriteSheetHandle, Texture, TextureMetadata,
    },
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};

pub struct Vpcap;

impl SimpleState for Vpcap{
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        world.register::<SimplePacket>();
        load_pcap(world);
    }
}

fn load_pcap(world: &mut World) {
    let mut cap = Capture::from_file("/home/eric/Downloads/test.pcap").expect("error opening pcap");

    while let Ok(p) = cap.next() {
        if let Some(ether_packet) = EthernetPacket::new(&p) {
            if let Some(pack) = handle_ethernet_frame(&ether_packet, &p.header.ts){
                world.create_entity()
                .with(pack)
                .build();
            }
        }
    }
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