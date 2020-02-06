//mod bundle;
mod simplenet;
//mod systems;
//mod util;
//mod vpcap;

//use crate::bundle::PacketBundle;
use crate::simplenet::*;

use chrono::naive::NaiveDateTime;
use chrono::Utc;
use std::net::IpAddr;
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use legion::prelude::*;
use ggez::nalgebra::{Point2, Rotation2, Vector2, Point, U1, U2, Matrix, ArrayStorage};

use ggez::{graphics, Context, ContextBuilder, GameResult};
use ggez::graphics::{Text, TextFragment, Scale, Color, Mesh};
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
    absolute: NaiveDateTime,
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
                    hosts.push(pack.get_dest_ip_addr());
                    packs.push(pack);
                }
            }
        }

        hosts.sort();
        hosts.dedup();

        let coords = graphics::screen_coordinates(ctx);
        let angle = Rotation2::new((360.0 / hosts.len() as f32).to_radians());
        let mut point = Point2::new(0.0_f32, 200.0_f32);
        let center = Vector2::new(coords.w / 2.0 - 50.0, coords.h / 2.0);
        let hosts: Vec<(IpAddr, Point<f32, U2>)> = hosts.into_iter().map(|host| {point = angle * point;(host,point + center)}).collect();

        let packs = packs.into_iter().map(|x| {
            let start = hosts.iter().filter(|host| host.0 == x.get_source_ip_addr()).collect::<Vec<&(IpAddr, Point<f32, U2>)>>()[0].1;
            let end = hosts.iter().filter(|host| host.0 == x.get_dest_ip_addr()).collect::<Vec<&(IpAddr, Point<f32, U2>)>>()[0].1;
            let vec2 = (start - end) / 5.0_f32;
            let color = match &x {
                SimplePacket::Ip(x) => match x.get_dest_port() {
                    Some(53) => Color::new(1., 0., 0., 1.),
                    Some(80) | Some(8080) | Some(443) => Color::new(0., 1., 0., 1.),
                    _ => Color::new(1., 1., 1., 1.),
                },
                SimplePacket::Arp(_) => Color::new(0., 0., 1., 1.),
            };
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Point2::new(0.0, 0.0),
                6.0,
                1.0,
                color,
            ).unwrap();
            (x, start, vec2, circle)
        });

        world.insert((), packs.into_iter());
        world.insert((), hosts.into_iter());

        let start = StartTime {
            start: start.unwrap(),
        };

        MyGame {
            start,
            world,
            absolute: Utc::now().naive_utc(),
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
        for (point, host) in query.iter(&mut self.world) {
            let text = Text::new(TextFragment::new(host.to_string()).scale(Scale{x: 12.0, y: 12.0}));
            graphics::draw(ctx, &text, (*point,))?;
        }
        let query = <(Read<SimplePacket>,Read<Point<f32,U2>>, Read<Matrix<f32, U2, U1,ArrayStorage<f32, U2, U1>>>, Read<Mesh>)>::query();
        for (pack, point, direction, circle) in query.iter(&mut self.world) {
            let time_delta = pack.get_ts() - self.start.start;
            let now = Utc::now().naive_utc() - self.absolute;
            let step = now - time_delta;
            if time_delta < now
            && step < chrono::Duration::seconds(5) {
                graphics::draw(ctx, &*circle, (*point - (*direction * step.to_std().unwrap().as_secs_f32()) ,))?;
            }
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
