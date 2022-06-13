mod denoise_plugin;
mod init;
mod orbit_controls;
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
        // .add_system(hello_world)
        .run();
}
