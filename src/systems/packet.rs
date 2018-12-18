use crate::simplenet::SimplePacket;

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
        //Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (packets, _time, hosts, mut debug_lines): Self::SystemData) {
        for packet in (&packets).join() {
            let source = packet.get_source_ip_addr().to_string();
            let dest = packet.get_dest_ip_addr().to_string();
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
            if end.len() > 0 {
                //println!(
                //    "drawing {} {} {} {}",
                //    start[0].0, start[0].1, end[0].0, end[0].1
                //);

                debug_lines.draw_line(
                    [start[0].0 / 1600. -1.2, start[0].1 / 900. + 0.1, 1.].into(),
                    [end[0].0 / 1600. -1.2, end[0].1 / 900. + 0.1, 1.].into(),
                    [0., 0., 0., 1.].into(),
                );
            }
        }
    }
}
