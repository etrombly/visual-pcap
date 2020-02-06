//mod bundle;
mod simplenet;
//mod systems;
//mod util;
//mod vpcap;

//use crate::bundle::PacketBundle;
use crate::simplenet::*;

use chrono::naive::NaiveDateTime;
use std::net::IpAddr;
use std::time::Duration;
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use legion::prelude::*;
use ggez::nalgebra::{Point2, Rotation2, Vector2};

use ggez::{graphics, Context, ContextBuilder, GameResult};
use ggez::graphics::{Text, TextFragment, Scale};
use ggez::event::{self, EventHandler};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Host(IpAddr);
fn main() {
    // Make a Context.
    let (mut ctx, mut event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
		.build()
		.expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = MyGame::new(&mut ctx);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}

struct MyGame {
    start: StartTime,
    world: World,
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        // Create a world to store our entities
        let universe = Universe::new();
        let mut world = universe.create_world();

        // Load/create resources such as images here.
        let mut cap = Capture::from_file("/home/eric/Downloads/test.pcap").expect("error opening pcap");
        let mut hosts = Vec::new();
        let mut start: Option<NaiveDateTime> = None;
        let mut packs = Vec::new();
    
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
                    /*
                    let mesh = create_mesh(world, generate_circle_vertices(2.0, 16));
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
                    */
                    packs.push((pack,));
                }
            }
        }

        world.insert((), packs.into_iter());

        hosts.sort();
        hosts.dedup();

        let coords = graphics::screen_coordinates(ctx);
        let angle = Rotation2::new((360.0 / hosts.len() as f32).to_radians());
        let mut point = Point2::new(0.0_f32, 200.0_f32);
        let center = Vector2::new(coords.w / 2.0 - 50.0, coords.h / 2.0);

        world.insert((), hosts.into_iter().map(|host| {point = angle * point;(host,point + center)}));

        let start = StartTime {
            start: start.unwrap(),
        };

        MyGame {
            start,
            world
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // Update code here...
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        let query = <(Read<Point2<f32>>,Read<IpAddr>)>::query();
        for (point, addr) in query.iter(&mut self.world) {
            let text = Text::new(TextFragment::new(addr.to_string()).scale(Scale{x: 12.0, y: 12.0}));
            graphics::draw(ctx, &text, (*point,))?;
        }
        graphics::present(ctx)
    }
}

pub struct StartTime {
    pub start: NaiveDateTime,
}

impl Default for StartTime {
    fn default() -> StartTime {
        StartTime {
            start: NaiveDateTime::from_timestamp(0, 0),
        }
    }
}
