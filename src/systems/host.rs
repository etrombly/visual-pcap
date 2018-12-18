use amethyst::{
    core::timing::Time,
    ecs::prelude::{Join, Read, ReadStorage, System},
    ui::UiText,
};

pub struct HostSystem;

impl<'s> System<'s> for HostSystem {
    type SystemData = (
        ReadStorage<'s, UiText>,
        Read<'s, Time>,
        //Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (hosts, _time): Self::SystemData) {
        for _host in (&hosts).join() {
            //println!("{:?}", host.text);
        }
    }
}
