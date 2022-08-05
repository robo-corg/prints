use super::Value;
use serde::{
    de::value::{MapDeserializer, SeqDeserializer, StrDeserializer},
    forward_to_deserialize_any,
};

pub type Error = serde::de::value::Error;

pub(crate) struct ValueDeserializer<'a> {
    pub value: &'a Value,
}

impl<'a> serde::de::IntoDeserializer<'a> for ValueDeserializer<'a> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> serde::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::String(val) => {
                visitor.visit_str(val.as_str())
            },
            Value::I32(val) => visitor.visit_i32(*val),
            Value::F32(val) => visitor.visit_f32(*val),
            Value::Vec(values) => visitor.visit_seq(SeqDeserializer::new(
                values.iter().map(|value| ValueDeserializer { value }),
            )),
            Value::KeyMap(map) => visitor.visit_map(MapDeserializer::new(
                map.iter()
                    .map(|(key, value)| (key.as_str(), ValueDeserializer { value })),
            )),
            Value::Entity(_comp_map) => {
                unimplemented!()
            }
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>, {
        match self.value {
            Value::String(val) => {
                visitor.visit_enum(StrDeserializer::new(val))
            },
            _other => {
                unimplemented!()
            }
        }
    }


    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}
