mod denoise_plugin;
mod half_edge;
mod init;
mod orbit_controls;
mod simplification_plugin;
mod simplificate;
use bevy::{asset::AssetServerSettings, prelude::*};
use denoise_plugin::DenoisePlugin;
use init::SurfaceDefaultPlugin;

// fn hello_world() {
//     println!("Hello World");
// }

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..Default::default()
        })
        .add_plugin(SurfaceDefaultPlugin)
        .add_plugin(DenoisePlugin)
        .run();
}
