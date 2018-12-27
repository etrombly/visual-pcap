mod bundle;
mod simplenet;
mod systems;
mod vpcap;

use crate::bundle::PacketBundle;
use crate::simplenet::*;

use std::net::IpAddr;
use std::time::Duration;
use chrono::naive::NaiveDateTime;

use amethyst::{
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    ecs::prelude::{Component, DenseVecStorage},
    input::InputBundle,
    prelude::*,
    renderer::{
        DisplayConfig, DrawDebugLines, DrawFlat2D, Pipeline, PosColorNorm, RenderBundle, Stage,
    },
    ui::{DrawUi, UiBundle},
    //utils::application_root_dir,
};

fn main() -> amethyst::Result<()> {
    use crate::vpcap::Vpcap;

    amethyst::start_logger(Default::default());
    //let app_root = application_root_dir();
    let display_config_path = format!("{}/resources/display.ron", ".");
    let config = DisplayConfig::load(&display_config_path);
    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([1.0, 1.0, 1.0, 1.0], 1.0)
            .with_pass(DrawFlat2D::new())
            .with_pass(DrawDebugLines::<PosColorNorm>::new())
            .with_pass(DrawUi::new()),
    );
    //let key_bindings_path = format!("{}/resources/input.ron", app_root);
    let key_bindings_path = format!("{}/resources/input.ron", ".");
    let assets_dir = "./src/assets";

    let game_data = GameDataBuilder::default()
        .with_bundle(PacketBundle)?
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with_bundle(TransformBundle::new().with_dep(&["packet_system", "host_system"]))?
        .with_bundle(UiBundle::<String, String>::new())?;
    let mut game = Application::build(assets_dir, Vpcap)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;
    game.run();
    Ok(())
}

pub struct StartTime {
    pub start: NaiveDateTime,
}

impl Default for StartTime {
    fn default () -> StartTime {
        StartTime {start: NaiveDateTime::from_timestamp(0, 0)}
    }
}

impl Component for SimplePacket {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug)]
pub struct IpAddrS(pub IpAddr);

impl Component for IpAddrS {
    type Storage = DenseVecStorage<Self>;
}
