//! Prints integration for bevy

use std::marker::PhantomData;

use bevy::app::{App, Plugin};
use bevy::asset::{AddAsset, AssetLoader, Assets, BoxedFuture, Handle, LoadContext, LoadedAsset};
use bevy::ecs::{
    entity::Entity,
    prelude::{Component, World},
    system::{Command, EntityCommands},
    world::EntityMut,
};

use bevy::utils::HashMap;
use serde::de::DeserializeOwned;
use tracing::info;

use crate::{
    bevy_prints::spawn::add_to_entity,
    expr::{Context, Evaluatable},
    runtime::SimpleRuntime,
    value::{EntityMap, Value},
    Blueprint, Error,
};

mod spawn;

/// Strategy for how add a [`crate::value::Value`] to an entity
pub trait ComponentAdder {
    fn add_to_entity(
        &self,
        world: &mut World,
        entity: Entity,
        component_name: String,
        component_value: Value,
    );
}

type DepInserter = Box<dyn for<'a> Fn(&'a mut EntityMut) + Sync + Send>;

pub struct ComponentInserter<T, D>
where
    for<'a> D: Fn(&'a mut World, String, Value) -> T,
{
    deserialize: D,
    deps: Vec<DepInserter>,
}

impl<T, D> ComponentAdder for ComponentInserter<T, D>
where
    for<'a> D: Fn(&'a mut World, String, Value) -> T,
    T: Component,
{
    fn add_to_entity(
        &self,
        world: &mut World,
        entity: Entity,
        component_name: String,
        component_value: Value,
    ) {
        info!(name = component_name.as_str(), "Adding component");
        let component: T = (self.deserialize)(world, component_name, component_value);
        let mut ent_mut = world.entity_mut(entity);

        ent_mut.insert(component);

        for dep in self.deps.iter() {
            dep(&mut ent_mut);
        }
    }
}

impl<T, D> ComponentInserter<T, D>
where
    for<'a> D: Fn(&'a mut World, String, Value) -> T,
{
    pub fn new(deserialize: D) -> Self {
        ComponentInserter {
            deserialize,
            deps: Vec::new(),
        }
    }

    pub fn depends_on<U>(mut self) -> Self
    where
        U: Component + Default,
    {
        self.deps.push(Box::new(|entity_mut| {
            entity_mut.insert::<U>(Default::default());
        }));
        self
    }

    pub fn map_component<F, U>(
        self,
        f: F,
    ) -> ComponentInserter<U, impl for<'a> Fn(&'a mut World, String, Value) -> U>
    where
        for<'a> F: Fn(&'a mut World, T) -> U,
    {
        let ComponentInserter { deserialize, deps } = self;

        ComponentInserter {
            deserialize: move |world, name, val| {
                let source_comp = deserialize(world, name, val);
                f(world, source_comp)
            },
            deps,
        }
    }
}

pub fn serde_component<T>(
) -> ComponentInserter<T, impl for<'a> Fn(&'a mut World, String, Value) -> T>
where
    T: DeserializeOwned,
{
    ComponentInserter::new(|_world, _component_name, value| value.to_component().unwrap())
}

/// Use serde to create component and add it
struct DeserializerComponentAdder<T>(PhantomData<T>);

impl<T> ComponentAdder for DeserializerComponentAdder<T>
where
    T: DeserializeOwned + Component,
{
    fn add_to_entity(
        &self,
        world: &mut World,
        entity: Entity,
        component_name: String,
        component_value: Value,
    ) {
        info!(name = component_name.as_str(), "Adding component");
        let component: T = component_value.to_component().unwrap();
        world.entity_mut(entity).insert(component);
    }
}

#[derive(Default)]
pub struct BlueprintComponentAdderRegistry {
    entries: HashMap<String, Box<dyn ComponentAdder + Send + Sync>>,
}

impl BlueprintComponentAdderRegistry {
    fn get_adder(&self, component_name: &str) -> Option<&(dyn ComponentAdder + Send + Sync)> {
        self.entries.get(component_name).map(|entry| entry.as_ref())
    }

    pub fn register_component<C>(&mut self, name: &str, component: C)
    where
        C: ComponentAdder + Send + Sync + 'static,
    {
        self.entries.insert(name.to_string(), Box::new(component));
    }

    pub fn register_component_deserializer<T>(&mut self, name: &str)
    where
        T: DeserializeOwned + Component,
    {
        self.entries.insert(
            name.to_string(),
            Box::new(DeserializerComponentAdder::<T>(PhantomData)),
        );
    }
}

#[derive(Default)]
pub struct BlueprintAssetLoader;

impl AssetLoader for BlueprintAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let blueprint = Blueprint::load_from_ron_bytes(load_context.path(), bytes)?;

            load_context.set_default_asset(LoadedAsset::new(blueprint));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bp.ron"]
    }
}

fn eval_blueprint(
    world: &mut World,
    blueprint_handle: Handle<Blueprint>,
) -> Result<EntityMap<Value>, Error> {
    let blueprints: &Assets<Blueprint> = world.get_resource().unwrap();

    let blueprint = blueprints.get(&blueprint_handle).unwrap();
    let runtime = SimpleRuntime::new();

    blueprint.eval_to_entity(&Context::new(&runtime))
}

pub struct PrintsPlugin;

impl Plugin for PrintsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_asset::<Blueprint>()
            .init_asset_loader::<BlueprintAssetLoader>();
    }
}

struct InsertBlueprintCommand {
    entity: Entity,
    blueprint: Handle<Blueprint>,
}

impl Command for InsertBlueprintCommand {
    fn write(self, world: &mut World) {
        info!("Blueprint insert");
        let ent = eval_blueprint(world, self.blueprint).unwrap();
        info!(data=?&ent, "Blueprint data");
        add_to_entity(world, self.entity, ent);
    }
}

pub trait BlueprintEntityCommandExt {
    fn insert_blueprint(&mut self, blueprint: Handle<Blueprint>) -> &mut Self;
}

impl<'w, 's, 'a> BlueprintEntityCommandExt for EntityCommands<'w, 's, 'a> {
    fn insert_blueprint(&mut self, blueprint: Handle<Blueprint>) -> &mut Self {
        let cmd = InsertBlueprintCommand {
            entity: self.id(),
            blueprint,
        };

        self.commands().add(cmd);

        self
    }
}

/// [`bevy_app::App`] helper methods for blueprints
pub trait BlueprintAppExt {
    fn register_blueprint_component<C>(&mut self, name: &str, component: C) -> &mut Self
    where
        C: ComponentAdder + Send + Sync + 'static;

    /// Register component deserializer for component of type `T`
    fn register_blueprint_component_deserializer<T>(&mut self, name: &str) -> &mut Self
    where
        T: DeserializeOwned + Component;
}

impl BlueprintAppExt for App {
    fn register_blueprint_component<C>(&mut self, name: &str, component: C) -> &mut Self
    where
        C: ComponentAdder + Send + Sync + 'static,
    {
        let mut registry = self
            .world
            .get_resource_or_insert_with(BlueprintComponentAdderRegistry::default);
        registry.register_component(name, component);
        self
    }

    fn register_blueprint_component_deserializer<T>(&mut self, name: &str) -> &mut Self
    where
        T: DeserializeOwned + Component,
    {
        let mut registry = self
            .world
            .get_resource_or_insert_with(BlueprintComponentAdderRegistry::default);
        registry.register_component_deserializer::<T>(name);
        self
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        cexpr::{Context, EntityExprBuilder, Expr},
        ecs::bevy::{add_to_entity, BlueprintEntityCommandExt, PrintsPlugin},
        runtime::SimpleRuntime,
        value::Value,
        Blueprint,
    };
    use bevy_app::App;
    use bevy_asset::{AssetPlugin, AssetServer, Assets, Handle};
    use bevy_core::CorePlugin;
    use bevy_log::LogPlugin;
    use bevy_reflect::Reflect;
    use serde::{Deserialize, Serialize};

    use bevy_ecs::{
        prelude::{Component, Res},
        reflect::ReflectComponent,
        system::{Commands, ResMut},
        world::World,
    };

    #[derive(Component, Reflect, Default, Debug, Deserialize, Serialize, PartialEq)]
    #[reflect(Component)]
    struct TestComp {
        x: f32,
        y: f32,
    }

    #[test]
    fn test_simple_spawn() {
        let mut app = App::new();

        app.add_plugin(CorePlugin)
            .add_plugin(AssetPlugin)
            .add_plugin(PrintsPlugin)
            .register_type::<TestComp>();

        let mut world = app.world;

        let entity_expr = EntityExprBuilder::new()
            .add_component(
                "TestComp",
                Expr::KeyMap(
                    vec![
                        ("x".to_string(), Expr::Constant(Value::F32(42.0))),
                        ("y".to_string(), Expr::Constant(Value::F32(42.0))),
                    ]
                    .into_iter()
                    .collect(),
                ),
            )
            .build();

        let ent = world.spawn().id();
        let runtime = SimpleRuntime::new();

        let entity_value = entity_expr.eval(&Context::new(&runtime)).unwrap();

        add_to_entity(&mut world, ent, entity_value);

        let test_comp = world.entity(ent).get::<TestComp>().unwrap();

        assert_eq!(test_comp, &TestComp { x: 42.0, y: 42.0 });
    }

    fn start_up_system(mut bps: ResMut<Assets<Blueprint>>, mut commands: Commands) {
        let entity_expr = EntityExprBuilder::new()
            .add_component(
                "TestComp",
                Expr::KeyMap(
                    vec![
                        ("x".to_string(), Expr::Constant(Value::F32(42.0))),
                        ("y".to_string(), Expr::Constant(Value::F32(42.0))),
                    ]
                    .into_iter()
                    .collect(),
                ),
            )
            .build();

        let bp = Blueprint::new("test_bp", entity_expr);

        let bp_handle = bps.add(bp);

        commands.spawn().insert_blueprint(bp_handle);
    }

    #[test]
    fn test_command_spawn() {
        let mut app = App::new();

        app.add_plugin(CorePlugin)
            .add_plugin(AssetPlugin)
            .add_plugin(PrintsPlugin)
            .register_type::<TestComp>();
        //let mut world = World::new();

        //let mut world = app.world;

        app.add_startup_system(start_up_system);

        app.update();

        // let ent = world.spawn().id();
        // let runtime = SimpleRuntime::new();

        // let entity_value = entity_expr.eval(&Context::new(&runtime)).unwrap();

        // add_to_entity(
        //     &mut world,
        //     ent,
        //     entity_value
        // );

        // let test_comp = world.entity(ent).get::<TestComp>().unwrap();

        // assert_eq!(test_comp, &TestComp { x: 42.0, y: 42.0 });
    }

    #[test]
    fn test_asset_load() {
        let mut app = App::new();

        app.add_plugin(CorePlugin)
            .add_plugin(LogPlugin)
            .add_plugin(AssetPlugin)
            .add_plugin(PrintsPlugin)
            .register_type::<TestComp>();

        let mut handle = Arc::new(Mutex::new(None));

        let handle_startup = handle.clone();

        app.add_startup_system(
            move |mut bps: ResMut<Assets<Blueprint>>,
                  asset_server: Res<AssetServer>,
                  mut commands: Commands| {
                let bp_handle: Handle<Blueprint> = asset_server.load("blueprints/test.bp.ron");

                {
                    let mut handle_mut = handle_startup.lock().unwrap();
                    *handle_mut = Some(bp_handle);
                }
            },
        );

        let mut ticks = 0;

        let loaded_blueprint = loop {
            app.update();

            let handle_lock = handle.lock().unwrap();

            let blueprints = app.world.get_resource::<Assets<Blueprint>>().unwrap();

            if let Some(handle) = handle_lock.as_ref() {
                //dbg!(handle);
                if let Some(loaded_blueprint) = blueprints.get(handle) {
                    break loaded_blueprint;
                }
            }

            ticks += 1;

            if ticks > 100 {
                panic!("Timeout waiting for blueprint asset to load");
            }
        };
    }
}
