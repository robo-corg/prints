use bevy::prelude::*;
use prints::Blueprint;
use prints::bevy_prints::{PrintsPlugin, serde_component, BlueprintAppExt, BlueprintEntityCommandExt};
use serde::Deserialize;

// Prints allows serde as a strategy for creating components.
// By customizing our deserializer we can infuence how the blueprint's
// data is translated into the actual component.
//
// In this case we can use `#[serde(transparent)]` to make components that are represent by a single
// scalar value such as a string or float
#[derive(Deserialize, Component)]
#[serde(transparent)]
struct Name(String);

#[derive(Deserialize, Component)]
#[serde(transparent)]
struct Hitpoints(f32);

// Vec and HashMap are also supported
#[derive(Deserialize, Component)]
#[serde(transparent)]
struct Attacks(Vec<Attack>);

// Bare tagged enums are supported and eventually
// tuple and struct enum variants will be supported
#[derive(Deserialize)]
enum Attack {
    FireBreath,
    Scratch,
    Bark
}

// See below, this is used to attach Handle<Scene> components to the entity
// which allows us to set the 3d model for our entity via Prints
#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct ChildSceneComponent(String);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add prints plugin, this adds support for `Blueprint` assets types and other support
        // resources, and systems to make Prints work with bevy.
        .add_plugin(PrintsPlugin)
        // By default Prints attempts to use `bevy::reflect` add components to entities
        // using a similar algorithm to that of `bevy::scene`, however this not all types
        // are supported with reflect, and serde can often be provide more a ergonomic
        // translation
        .register_blueprint_component_deserializer::<Name>("Name")
        .register_blueprint_component_deserializer::<Hitpoints>("Hitpoints")
        .register_blueprint_component_deserializer::<Attacks>("Attacks")

        // Complex components can be added that have access to &mut World which is useful escape hatch
        // for shoring up missing prints functionality.
        //
        // In this case we can make a special component exposed as "Scene" represented as string path to
        // a scene resource, which is then loaded and the resulting Handle<Scene> is attached instead of
        // ChildSceneComponent.
        //
        // Eventually we should have first class support asset and builtin components like Handle<Scene>,
        // but until then we can work around it.
        .register_blueprint_component(
            "Scene",
            serde_component::<ChildSceneComponent>()
                .map_component(|world, comp| {
                    let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
                    let scene_handle: Handle<Scene> = asset_server.load(comp.0.as_str());
                    scene_handle
                })
                // Scene spawning expects a bundle that includes a couple other components. We can tell prints
                // to add default components for these if they are not already inserted yet.
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
    // Load our blueprint
    let corgi_blueprint: Handle<Blueprint> = asset_server.load("blueprints/corgi.bp.ron");

    commands
        .spawn()
        .insert_bundle(TransformBundle::default())
        // evaluate our blueprint and insert the components to this new entity.
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