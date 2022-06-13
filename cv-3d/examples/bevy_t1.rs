use bevy::{asset::AssetServerSettings, prelude::*};
use bevy_inspector_egui::{
    Inspectable, InspectorPlugin, RegisterInspectable, WorldInspectorPlugin,
};
use bevy_obj::*;

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Elaina Proctor".to_string()));
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Renzo Hume".to_string()));
    commands
        .spawn()
        .insert(Person)
        .insert(Name("Zayna Nieves".to_string()));
}

fn hello_world() {
    println!("Hello World")
}

// fn greet_people(query: Query<&Name, With<Person>>) {
//     for name in query.iter() {
//         println!("hello {}", name.0)
//     }
// }

struct GreetTimer(Timer);
fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in query.iter() {
            println!("hello {}", name.0)
        }
    }
}

struct HelloPlugin;
impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, true)))
            .add_startup_system(add_people)
            .add_system(greet_people);
    }
}

fn setup_load_obj_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: asset_server.load("stanford-bunny.obj"),
        ..Default::default()
    });
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(3.0, 4.0, 3.0)),
        ..Default::default()
    });

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(1.5, 2.7, 4.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    });
}

struct DenoiseScenePlugin;
impl Plugin for DenoiseScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_load_obj_scene);
    }
}

#[derive(Inspectable)]
struct Data {
    #[inspectable(min = 10.0, max = 70.0)]
    font_size: f32,
    text: String,
    show_square: bool,
    color: Color,
    #[inspectable(visual, min = Vec2::new(-200., -200.), max = Vec2::new(200., 200.))]
    position: Vec2,
    list: Vec<f32>,
    #[inspectable(replacement = String::default as fn() -> _)]
    option: Option<String>,
}
impl Default for Data {
    fn default() -> Self {
        Data {
            font_size: 50.0,
            text: "Hello World!".to_string(),
            show_square: true,
            color: Color::BLUE,
            position: Vec2::default(),
            list: vec![0.0],
            option: None,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(HelloPlugin)
        .add_plugin(ObjPlugin)
        .add_plugin(DenoiseScenePlugin)
        // .add_plugin(LookTransformPlugin)
        .add_plugin(InspectorPlugin::<Data>::new())
        // .add_system(move_camera_system)
        .run();
}
