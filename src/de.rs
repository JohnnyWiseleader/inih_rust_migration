use serde::de::{self, Deserialize, Deserializer, IntoDeserializer, MapAccess, Visitor};
use std::collections::hash_map;
use std::collections::HashMap;

use crate::parse_ini::{parse_ini_with_config, IniParserConfig};

/// Deserialize the INI file into a struct
pub fn from_ini_file<'de, T>(filename: &str, config: &IniParserConfig) -> Result<T, std::io::Error>
where
    T: Deserialize<'de>,
{
    let map = parse_ini_with_config(filename, config)?;
    println!("{:#?}", map);

    let deser = IniNestedDeserializer { map }; // <-- nested
    T::deserialize(deser).map_err(|e| {
        eprintln!("Deserialization error: {:?}", e);
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Deserialization failed: {e}"),
        )
    })
}

pub struct IniNestedDeserializer {
    pub map: HashMap<String, HashMap<String, String>>,
}

impl<'de> Deserializer<'de> for IniNestedDeserializer {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(IniNestedMapAccess {
            iter: self.map.into_iter(),
            next_value: None,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string unit seq tuple
        tuple_struct map struct enum identifier ignored_any bytes byte_buf option
        unit_struct newtype_struct
    }
}

pub struct IniNestedMapAccess {
    iter: hash_map::IntoIter<String, HashMap<String, String>>,
    next_value: Option<HashMap<String, String>>,
}

impl<'de> MapAccess<'de> for IniNestedMapAccess {
    type Error = de::value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some((k, v)) = self.iter.next() {
            self.next_value = Some(v);
            seed.deserialize(k.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(v_map) = self.next_value.take() {
            use serde::de::value::StringDeserializer;

            let iter = v_map.into_iter().map(|(k, v)| {
                // Deserializer needs to be StringDeserializer so it can be converted to any value
                let value = StringDeserializer::<Self::Error>::new(v);
                (k, value)
            });

            let map_de = serde::de::value::MapDeserializer::new(iter);
            seed.deserialize(map_de)
        } else {
            Err(de::Error::custom("Missing value for key"))
        }
    }
}
