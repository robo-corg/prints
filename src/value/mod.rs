//! Evaluated prints expressions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod deser;

pub use deser::Error as ToComponentError;

#[derive(PartialEq, Debug, Clone, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct EntityMap<T>(HashMap<String, T>);

impl<T> EntityMap<T> {
    pub fn new() -> Self {
        EntityMap(HashMap::new())
    }

    pub fn add_component(&mut self, name: &str, value: T) {
        self.0.insert(name.to_string(), value);
    }

    pub fn components(&self) -> impl Iterator<Item = (&str, &T)> {
        self.0.iter().map(|(name, value)| (name.as_str(), value))
    }

    pub fn into_components(self) -> impl Iterator<Item = (String, T)> {
        self.0.into_iter()
    }

    pub fn try_map<F, U, E>(self, mut f: F) -> Result<EntityMap<U>, E>
    where
        F: FnMut(T) -> Result<U, E>,
    {
        Ok(EntityMap(
            self.0
                .into_iter()
                .map(|(name, comp)| Ok((name, f(comp)?)))
                .collect::<Result<_, E>>()?,
        ))
    }

    pub fn map<F, U>(self, mut f: F) -> EntityMap<U>
    where
        F: FnMut(T) -> U,
    {
        EntityMap(
            self.0
                .into_iter()
                .map(|(name, comp)| (name, f(comp)))
                .collect(),
        )
    }
}

// impl EntityMap<Value> {
//     pub fn to_entity<EntityBuilderType>(
//         &self,
//         comp_lib: &CompLibrary<EntityBuilderType>,
//         entity: &mut EntityBuilderType,
//     ) -> Result<(), Error>
//     where
//         EntityBuilderType: EntityBuilder,
//     {
//         for (name, comp_value) in self.0.iter() {
//             let comp_builder = comp_lib
//                 .get_builder(name)
//                 .ok_or_else(|| Error::UnknownComponent(name.to_string()))?;

//             comp_builder(entity, Cow::Borrowed(comp_value))?;
//         }

//         Ok(())
//     }
// }

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    KeyMap(HashMap<String, Value>),
    String(String),
    I32(i32),
    F32(f32),
    Vec(Vec<Value>),
    Entity(EntityMap<Value>),
}

impl Value {
    pub fn to_component<'a, T>(&'a self) -> Result<T, ToComponentError>
    where
        T: Deserialize<'a>,
    {
        let d = deser::ValueDeserializer { value: self };
        T::deserialize(d)
    }

    pub fn typename(&self) -> &'static str {
        match self {
            Value::KeyMap(_) => "map",
            Value::String(_) => "string",
            Value::I32(_) => "i32",
            Value::F32(_) => "f32",
            Value::Vec(_) => "vec",
            Value::Entity(_) => "entity",
        }
    }
}
