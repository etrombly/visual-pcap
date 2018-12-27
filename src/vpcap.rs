use crate::simplenet::*;
use crate::IpAddrS;
use crate::StartTime;
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use chrono::naive::NaiveDateTime;

use amethyst::{
    assets::Loader,
    core::{
        nalgebra::{Point2, Rotation2},
        transform::Transform,
    },
    ecs::{
        prelude::World,
    },
    prelude::*,
    renderer::*,
    renderer::{Camera, Projection},
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};

pub struct Vpcap;

impl SimpleState for Vpcap {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        world.add_resource(DebugLines::new().with_capacity(100));
        world.add_resource(DebugLinesParams {
            line_width: 1.0 / 400.0,
        });

        world.register::<SimplePacket>();
        world.register::<IpAddrS>();
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
        .with(Camera::from(Projection::perspective(
                1.33333,
                std::f32::consts::FRAC_PI_2,
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
                }else{
                    start = Some(pack.get_ts());
                }
                hosts.push(pack.get_source_ip_addr());
                world.create_entity().with(pack).build();
            }
        }
    }
    println!("{:?}", start);
    let start = StartTime { start: start.unwrap()};
    world.add_resource(start);

    hosts.sort();
    hosts.dedup();

    let angle = Rotation2::new((360.0 / hosts.len() as f32).to_radians());
    let mut last_point = Point2::new(0., 600.);
    for host in &hosts {
        world.create_entity().with(IpAddrS(*host)).build();

        last_point = angle * last_point;

        let host_transform = UiTransform::new(
            host.to_string(),
            Anchor::Middle,
            last_point.coords.x + 900.,
            last_point.coords.y - 600.,
            1.,
            200.,
            50.,
            0,
        );

        let _host_text = world
            .create_entity()
            .with(host_transform)
            .with(UiText::new(
                font.clone(),
                host.to_string(),
                [0., 0., 0., 1.],
                20.,
            ))
            .build();
    }
}
