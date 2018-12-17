use crate::simplenet::*;
use crate::IpAddrS;
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use std::net::IpAddr;

use amethyst::{ecs::prelude::{World, Join},
               core::{
                   transform::Transform,
                   nalgebra::{Point2, Rotation2},
               },
               renderer::{
                Camera, Flipped, PngFormat, Projection, SpriteRender, SpriteSheet, SpriteSheetFormat,
                SpriteSheetHandle, Texture, TextureMetadata,
               },               
               assets::{AssetStorage, Loader},
               ui::{Anchor, TtfFormat, UiText, UiTransform},
               prelude::*};

pub struct Vpcap;

impl SimpleState for Vpcap {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        world.register::<SimplePacket>();
        world.register::<IpAddrS>();
        initialise_camera(world);
        load_pcap(world);
    }
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_z(1.0);
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            900.0,
            900.0,
            0.0,
        )))
        .with(transform)
        .build();
}

fn load_pcap(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/FiraSans-Regular.ttf",
        TtfFormat,
        Default::default(),
        (),
        &world.read_resource(),
    );

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

    let angle = Rotation2::new((360.0 / hosts.len() as f32).to_radians());
    println!("{}", angle);
    let mut last_point = Point2::new(0.,600.);
    for host in &hosts {
        world.create_entity().with(IpAddrS(*host)).build();

        last_point     = angle * last_point;
        println!("{}", last_point);

        let host_transform = UiTransform::new(
            host.to_string(),
            Anchor::Middle,
            last_point.coords.x + 200.,
            last_point.coords.y - 600.,
            1.,
            200.,
            50.,
            0,
        );

        let host_text = world
            .create_entity()
            .with(host_transform)
            .with(UiText::new(
                font.clone(),
                host.to_string(),
                [0., 0., 0., 1.],
                25.,
            ))
            .build();
    }
}
