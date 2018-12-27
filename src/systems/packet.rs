use crate::simplenet::SimplePacket;
use crate::StartTime;

use amethyst::{
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System, Write},
    renderer::*,
    ui::UiTransform,
};

pub struct PacketSystem;

impl<'s> System<'s> for PacketSystem {
    type SystemData = (
        ReadStorage<'s, SimplePacket>,
        Read<'s, Time>,
        ReadStorage<'s, UiTransform>,
        Write<'s, DebugLines>,
        Read<'s, StartTime>,
        //Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (packets, time, hosts, mut debug_lines, start): Self::SystemData) {
        for packet in (&packets).join() {
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
            if end.len() > 0 && time_delta < now && now - time_delta < std::time::Duration::new(2, 0){
                debug_lines.draw_line(
                    [start[0].0 / 1600. -1., start[0].1 / 900. + 0.3, 1.].into(),
                    [end[0].0 / 1600. -1., end[0].1 / 900. + 0.3, 1.].into(),
                    [0., 0., 0., 1.].into(),
                );
            }
        }
    }
}
