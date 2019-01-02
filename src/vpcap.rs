use crate::simplenet::*;
use crate::util::*;
use crate::IpAddrS;
use crate::StartTime;

use chrono::naive::NaiveDateTime;
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;

use amethyst::{
    assets::Loader,
    core::{
        nalgebra::{Point2, Rotation2},
        transform::Transform,
    },
    ecs::prelude::World,
    prelude::*,
    renderer::*,
    renderer::{Camera, Projection},
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};

pub struct Vpcap;

impl SimpleState for Vpcap {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        world.register::<SimplePacket>();
        world.register::<IpAddrS>();
        world.register::<Material>();
        initialise_camera(world);
        load_pcap(world);
    }
}

/// Initialise the camera.
fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_z(2.0);
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0, 600.0, 0.0, 600.0,
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
    let mut start: Option<NaiveDateTime> = None;

    while let Ok(p) = cap.next() {
        if let Some(ether_packet) = EthernetPacket::new(&p) {
            if let Some(pack) = handle_ethernet_frame(&ether_packet, &p.header.ts) {
                if let Some(time) = start {
                    if pack.get_ts() < time {
                        start = Some(pack.get_ts());
                    }
                } else {
                    start = Some(pack.get_ts());
                }
                hosts.push(pack.get_source_ip_addr());
                // Create the mesh, material and translation.
                let color = match &pack {
                    SimplePacket::Ip(x) => match x.get_dest_port() {
                        Some(53) => [1., 0., 0., 1.],
                        Some(80) | Some(8080) | Some(443) => [0., 1., 0., 1.],
                        _ => [1., 1., 1., 1.],
                    },
                    SimplePacket::Arp(_) => [0., 0., 1., 1.],
                };
                let mesh = create_mesh(world, generate_circle_vertices(3.0, 16));
                let material = create_color_material(world, color);
                let mut local_transform = Transform::default();
                local_transform.set_position([300.0, 300.0, 1.0].into());
                world
                    .create_entity()
                    .with(mesh)
                    .with(material)
                    .with(pack)
                    .with(local_transform)
                    .build();
            }
        }
    }

    let start = StartTime {
        start: start.unwrap(),
    };
    world.add_resource(start);

    hosts.sort();
    hosts.dedup();

    let angle = Rotation2::new((360.0 / hosts.len() as f32).to_radians());
    let mut point = Point2::new(0., 590.0);
    for host in &hosts {
        world.create_entity().with(IpAddrS(*host)).build();
        
        point = angle * point;

        let ui_transform = UiTransform::new(
            host.to_string(),
            Anchor::Middle,
            point.x + 1200.0,
            point.y - 600.0,
            1.,
            100.,
            50.,
            0,
        );

        let _host_text = world
            .create_entity()
            .with(ui_transform)
            .with(UiText::new(
                font.clone(),
                host.to_string(),
                [1., 0., 0., 1.],
                20.,
            ))
            .build();
    }
}
