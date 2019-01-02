use crate::simplenet::SimplePacket;
use crate::StartTime;

use amethyst::{
    core::{timing::Time, transform::Transform},
    ecs::prelude::{Join, Read, ReadStorage, System, Write, WriteStorage},
    renderer::*,
    ui::UiTransform,
};

pub struct PacketSystem;

impl<'s> System<'s> for PacketSystem {
    type SystemData = (
        ReadStorage<'s, SimplePacket>,
        Read<'s, Time>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, UiTransform>,
        Read<'s, StartTime>,
        //Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (packets, time, mut trans, hosts, start): Self::SystemData) {
        for (packet, trans) in (&packets, &mut trans).join() {
            let source = packet.get_source_ip_addr().to_string();
            let dest = packet.get_dest_ip_addr().to_string();
            let time_delta = (packet.get_ts() - start.start).to_std().unwrap();
            let now = time.absolute_time();
            let start: Vec<(_, _)> = hosts
                .join()
                .filter_map(|x| {
                    if source == x.id {
                        Some((x.pixel_x(), x.pixel_y()))
                    } else {
                        None
                    }
                })
                .collect();
            let end: Vec<(_, _)> = hosts
                .join()
                .filter_map(|x| {
                    if dest == x.id {
                        Some((x.pixel_x(), x.pixel_y()))
                    } else {
                        None
                    }
                })
                .collect();
            if end.len() > 0
                && time_delta < now
                && now - time_delta < std::time::Duration::new(10, 0)
            {
                let delta_x =
                    ((start[0].0 - end[0].0) / 10.0) * (now - time_delta).as_float_secs() as f32;
                let delta_y =
                    ((start[0].1 - end[0].1) / 10.0) * (now - time_delta).as_float_secs() as f32;
                trans.set_position(
                    [
                        (start[0].0 - 100.0 - delta_x) / 5.0,
                        (start[0].1 + 1170.0 - delta_y) / 3.0,
                        1.,
                    ]
                    .into(),
                );
            }
        }
    }
}
