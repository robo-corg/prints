use bevy::prelude::*;
use prints::Blueprint;
use prints::bevy_prints::{PrintsPlugin, serde_component, BlueprintAppExt, BlueprintEntityCommandExt};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct ChildSceneComponent(String);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PrintsPlugin)
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
) {
    let corgi_blueprint: Handle<Blueprint> = asset_server.load("blueprints/corgi.bp.ron");

    commands
        .spawn()
        .insert_bundle(TransformBundle::default())
        //.insert(ChildScene::new("models/warrior_01_no_amin.glb#Scene0"));
        .insert_blueprint(corgi_blueprint);

}