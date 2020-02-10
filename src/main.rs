//mod bundle;
mod simplenet;
//mod systems;
//mod util;
//mod vpcap;

//use crate::bundle::PacketBundle;
use crate::simplenet::*;

use chrono::naive::NaiveDateTime;
use chrono::Utc;
use generational_arena::Index;
use ggez::nalgebra::{ArrayStorage, Matrix, Point, Point2, Rotation2, Vector2, U1, U2};
use legion::prelude::*;
use ncollide2d::query;
use ncollide2d::shape::Ball;
use nphysics2d::algebra::{ForceType, Force2};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::math::Force;
use nphysics2d::object::{
    BodyPartHandle, BodyStatus, DefaultBodyHandle, DefaultBodyPartHandle, DefaultBodySet,
    DefaultColliderHandle, DefaultColliderSet, RigidBody, RigidBodyDesc,
};
use nphysics2d::world::{
    DefaultGeometricalWorld, DefaultMechanicalWorld, GeometricalWorld, MechanicalWorld,
};
use pcap::Capture;
use pnet::packet::ethernet::EthernetPacket;
use std::net::IpAddr;
use rayon::prelude::*;

use ggez::event::{self, EventHandler};
use ggez::graphics::{Color, Mesh, Scale, Text, TextFragment};
use ggez::{graphics, Context, ContextBuilder, GameResult};

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
        Err(e) => println!("Error occured: {}", e),
    }
}

struct MyGame {
    start: StartTime,
    absolute: NaiveDateTime,
    last: NaiveDateTime,
    world: World,
    mworld: MechanicalWorld<f32, DefaultBodySet<f32>, Index>,
    gworld: GeometricalWorld<f32, DefaultBodyHandle, DefaultColliderHandle>,
    bodies: DefaultBodySet<f32>,
    colliders: DefaultColliderSet<f32>,
    jcon: DefaultJointConstraintSet<f32>,
    fgen: DefaultForceGeneratorSet<f32>,
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        // Create a world to store our entities
        let universe = Universe::new();
        let mut world = universe.create_world();
        let coords = graphics::screen_coordinates(ctx);

        let mut mechanical_world = DefaultMechanicalWorld::new(Vector2::new(coords.w, coords.h));
        let mut geometrical_world = DefaultGeometricalWorld::new();

        let mut bodies = DefaultBodySet::new();
        let mut colliders = DefaultColliderSet::new();
        let mut joint_constraints = DefaultJointConstraintSet::new();
        let mut force_generators = DefaultForceGeneratorSet::new();

        // step is just so the world can infer types
        mechanical_world.step(
            &mut geometrical_world,
            &mut bodies,
            &mut colliders,
            &mut joint_constraints,
            &mut force_generators,
        );

        // Load/create resources such as images here.
        let mut cap =
            Capture::from_file("/home/eric/Downloads/test.pcap").expect("error opening pcap");
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
        let hosts: Vec<(IpAddr, Point<f32, U2>)> = hosts
            .into_iter()
            .map(|host| {
                point = angle * point;
                let body = RigidBodyDesc::new()
                    .status(BodyStatus::Static)
                    .translation(Vector2::new(point.x, point.y))
                    .build();
                bodies.insert(body);
                (host, point + center)
            })
            .collect();

        let packs = packs.into_iter().map(|x| {
            let start = hosts
                .iter()
                .filter(|host| host.0 == x.get_source_ip_addr())
                .collect::<Vec<&(IpAddr, Point<f32, U2>)>>()[0]
                .1;
            let end = hosts
                .iter()
                .filter(|host| host.0 == x.get_dest_ip_addr())
                .collect::<Vec<&(IpAddr, Point<f32, U2>)>>()[0]
                .1;
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
            )
            .unwrap();
            (x, start, Vector2::new(end.x, end.y), circle)
        });

        world.insert((), packs);
        world.insert((), hosts.into_iter());

        let start = StartTime {
            start: start.unwrap(),
        };

        MyGame {
            start,
            world,
            mworld: mechanical_world,
            gworld: geometrical_world,
            bodies,
            colliders,
            jcon: joint_constraints,
            fgen: force_generators,
            absolute: Utc::now().naive_utc(),
            last: Utc::now().naive_utc(),
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
        let query = <(Read<Point2<f32>>, Read<IpAddr>)>::query();
        for (point, host) in query.iter(&mut self.world) {
            let text =
                Text::new(TextFragment::new(host.to_string()).scale(Scale { x: 12.0, y: 12.0 }));
            graphics::draw(ctx, &text, (*point,))?;
        }
        let now = Utc::now().naive_utc() - self.absolute;
        self.mworld.set_timestep(
            (Utc::now().naive_utc() - self.last)
                .to_std()
                .unwrap()
                .as_secs_f32(),
        );
        self.mworld.step(
            &mut self.gworld,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.jcon,
            &mut self.fgen,
        );
        let query = <(Read<SimplePacket>, Read<Point<f32, U2>>)>::query()
            .filter(!component::<DefaultBodyHandle>());
        let mut updates = Vec::new();
        for (entity, (pack, point)) in query.iter_entities(&mut self.world) {
            let time_delta = pack.get_ts() - self.start.start;
            let step = now - time_delta;
            if time_delta < now && step < chrono::Duration::seconds(5) {
                let body = RigidBodyDesc::new()
                    .translation(Vector2::new(point.x, point.y))
                    .gravity_enabled(false)
                    .mass(0.1)
                    .build();
                let handle = self.bodies.insert(body);
                updates.push((entity, handle));
            }
        }
        for update in updates {
            self.world.add_component(update.0, update.1);
        }
        let query = <(
            Read<SimplePacket>,
            Read<Matrix<f32, U2, U1, ArrayStorage<f32, U2, U1>>>,
            Read<Mesh>,
            Read<DefaultBodyHandle>,
        )>::query();
        let mut to_drop = Vec::new();
        let mut active = Vec::new();

        for (entity, (pack, direction, circle, handle)) in query.iter_entities(&mut self.world) {
            let time_delta = pack.get_ts() - self.start.start;
            let step = now - time_delta;
            //if step < chrono::Duration::seconds(5) {
                active.push(*handle);
                let body = self.bodies.get_mut(*handle).unwrap();
                let pos = body.part(0).unwrap().position().translation;
                let point = Point2::new(pos.x, pos.y);
                let delta_pos = point - *direction;
                if delta_pos < Point2::new(3., 3.) {
                    body.set_status(BodyStatus::Static);
                } else {
                    let force = Force::linear(Vector2::new(delta_pos.x, delta_pos.y) * -1.0);
                    body.apply_force(0, &force, ForceType::Force, true);
                }
                graphics::draw(ctx, &*circle, (point,))?;
            //} else {
                //self.bodies.remove(*handle);
                //to_drop.push(entity);
            //}
        }

        while let Some(current) = active.pop() {
            let body = self.bodies.get(current).unwrap();
            let current_pos = body.part(0).unwrap().position();
            let current_vel = body.part(0).unwrap().velocity();
            let results:Vec<Force2<f32>> = active.par_iter().filter_map(|other| {
                if self.bodies.get(*other).unwrap().status() != BodyStatus::Static {
                    let other_iso = self.bodies.get(*other).unwrap().part(0).unwrap().position();
                    let other_vel = self
                        .bodies
                        .get(*other)
                        .unwrap()
                        .part(0)
                        .unwrap()
                        .velocity()
                        .linear;
                    
                    let result = query::time_of_impact(
                        &current_pos,
                        &current_vel.linear,
                        &Ball::new(6.0),
                        &other_iso,
                        &other_vel,
                        &Ball::new(6.0),
                        10.0,
                        30.0,
                    );
                    if let Some(x) = result {
                        if x.toi < 0.07 && x.toi > 0.0 {
                            return Some(Force::linear((current_vel).rotated(&Rotation2::new(20.0)).linear))
                        }
                    }
                }
                None
            }).collect();
            if results.len() > 0 {
                let body = self.bodies.get_mut(current).unwrap();
                body.apply_force(0, &results[0], ForceType::Force, true);
            }
        }

        for update in to_drop {
            self.world.remove_component::<DefaultBodyHandle>(update);
        }
        self.last = Utc::now().naive_utc();
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
