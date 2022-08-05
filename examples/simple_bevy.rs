use bevy::prelude::*;
use prints::Blueprint;
use prints::bevy_prints::{PrintsPlugin, serde_component, BlueprintAppExt, BlueprintEntityCommandExt};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct ChildSceneComponent(String);

#[derive(Deserialize, Component)]
#[serde(transparent)]
struct Attacks(Vec<Attack>);

#[derive(Deserialize)]
enum Attack {
    FireBreath,
    Scratch,
    Bark
}

#[derive(Deserialize, Component)]
#[serde(transparent)]
struct Name(String);

#[derive(Deserialize, Component)]
#[serde(transparent)]
struct Hitpoints(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PrintsPlugin)
        .register_blueprint_component_deserializer::<Name>("Name")
        .register_blueprint_component_deserializer::<Hitpoints>("Hitpoints")
        .register_blueprint_component_deserializer::<Attacks>("Attacks")
        .register_blueprint_component(
            "Scene",
            serde_component::<ChildSceneComponent>()
                .map_component(|world, comp| {
                    let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
                    let scene_handle: Handle<Scene> = asset_server.load(comp.0.as_str());
                    scene_handle
                })
                .depends_on::<Transform>()
                .depends_on::<Visibility>()
                .depends_on::<ComputedVisibility>(),
        )
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    blueprints: Res<Assets<Blueprint>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let corgi_blueprint: Handle<Blueprint> = asset_server.load("blueprints/corgi.bp.ron");

    commands
        .spawn()
        .insert_bundle(TransformBundle::default())
        //.insert(ChildScene::new("models/warrior_01_no_amin.glb#Scene0"));
        .insert_blueprint(corgi_blueprint)
        ;

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}