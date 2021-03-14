use bevy::ecs::{
    entity::Entity,
    prelude::{Mut, ReflectComponent, World},
};
use bevy::reflect::{
    DynamicStruct, DynamicTupleStruct, Reflect, TupleStruct, TypeRegistryInternal,
};

use crate::{
    bevy_prints::BlueprintComponentAdderRegistry,
    value::{EntityMap, Value},
};

fn add_component_with_reflect(
    type_registry: &TypeRegistryInternal,
    world: &mut World,
    entity: Entity,
    component_name: String,
    component_value: Value,
) {
    let registration = match type_registry.get_with_short_name(&component_name) {
        Some(registration) => registration,
        None => {
            panic!(
                "Could not find component named `{}` in the type registry.",
                component_name
            );
        }
    };

    let reflect_component = registration.data::<ReflectComponent>().unwrap();

    let component: Box<dyn Reflect> = match component_value {
        Value::KeyMap(fields) => {
            let mut component = Box::new(DynamicStruct::default());

            component.set_name(component_name);

            for (key, value) in fields.iter() {
                //component.field_mut(key).

                match value {
                    Value::KeyMap(_) => todo!(),
                    Value::String(s) => component.insert(key, s.clone()),
                    Value::I32(v) => component.insert(key, *v),
                    Value::F32(v) => component.insert(key, *v),
                    Value::Vec(_v) => todo!(),
                    Value::Entity(_) => todo!(),
                }
            }

            component
        }
        Value::String(_) => todo!(),
        Value::I32(_) => todo!(),
        Value::F32(_) => todo!(),
        Value::Vec(vec) => {
            let mut component = Box::new(DynamicTupleStruct::default());

            component.set_name(component_name);

            for (field_index, value) in vec.into_iter().enumerate() {
                match value {
                    Value::KeyMap(_) => todo!(),
                    Value::String(s) => component.field_mut(field_index).unwrap().apply(&s), //component.insert(key, s.clone()),
                    Value::I32(v) => component.field_mut(field_index).unwrap().apply(&v),
                    Value::F32(v) => component.field_mut(field_index).unwrap().apply(&v),
                    Value::Vec(_v) => todo!(),
                    Value::Entity(_) => todo!(),
                }
            }

            component
        }
        Value::Entity(_) => todo!(),
    };

    reflect_component.apply_or_insert(world, entity, &*component);
}

pub(crate) fn add_to_entity(world: &mut World, entity: Entity, entity_value: EntityMap<Value>) {
    let type_registry = {
        world
            .get_resource::<bevy::reflect::TypeRegistryArc>()
            .unwrap()
            .clone()
    };

    let type_registry = type_registry.read();

    world.get_resource_or_insert_with(BlueprintComponentAdderRegistry::default);

    world.resource_scope(
        |world, adder_registry: Mut<BlueprintComponentAdderRegistry>| {
            for (component_name, component_value) in entity_value.into_components() {
                if let Some(adder) = adder_registry.get_adder(&component_name) {
                    adder.add_to_entity(world, entity, component_name, component_value);
                } else {
                    add_component_with_reflect(
                        &type_registry,
                        world,
                        entity,
                        component_name,
                        component_value,
                    );
                }
            }
        },
    );
}
