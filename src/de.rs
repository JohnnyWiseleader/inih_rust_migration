use serde::de::{
    self, Deserialize, Deserializer, IntoDeserializer, MapAccess, Unexpected, Visitor,
};
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

    let deser = IniNestedDeserializer { map };
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
            let iter = v_map
                .into_iter()
                .map(|(k, v)| (k, IniValueDeserializer::new(v).into_deserializer()));
            let map_de = serde::de::value::MapDeserializer::new(iter);
            seed.deserialize(map_de)
        } else {
            Err(de::Error::custom("Missing value for key"))
        }
    }
}

/// Custom deserializer for individual INI values
pub struct IniValueDeserializer {
    value: String,
}

impl IniValueDeserializer {
    pub fn new(value: String) -> Self {
        IniValueDeserializer { value }
    }
}

impl<'de> Deserializer<'de> for IniValueDeserializer {
    type Error = de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.value)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value.to_lowercase().as_str() {
            "true" | "yes" | "1" => visitor.visit_bool(true),
            "false" | "no" | "0" => visitor.visit_bool(false),
            _ => Err(de::Error::invalid_value(
                Unexpected::Str(&self.value),
                &"a boolean",
            )),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.value
            .parse::<i32>()
            .map_err(|_| de::Error::invalid_value(Unexpected::Str(&self.value), &"an integer"))
            .and_then(|v| visitor.visit_i32(v))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.value
            .parse::<i64>()
            .map_err(|_| de::Error::invalid_value(Unexpected::Str(&self.value), &"an integer"))
            .and_then(|v| visitor.visit_i64(v))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.value
            .parse::<u16>()
            .map_err(|_| {
                de::Error::invalid_value(Unexpected::Str(&self.value), &"an unsigned integer")
            })
            .and_then(|v| visitor.visit_u16(v))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.value
            .parse::<u32>()
            .map_err(|_| {
                de::Error::invalid_value(Unexpected::Str(&self.value), &"an unsigned integer")
            })
            .and_then(|v| visitor.visit_u32(v))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.value
            .parse::<u64>()
            .map_err(|_| {
                de::Error::invalid_value(Unexpected::Str(&self.value), &"an unsigned integer")
            })
            .and_then(|v| visitor.visit_u64(v))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.value
            .parse::<f32>()
            .map_err(|_| de::Error::invalid_value(Unexpected::Str(&self.value), &"a float"))
            .and_then(|v| visitor.visit_f32(v))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.value
            .parse::<f64>()
            .map_err(|_| de::Error::invalid_value(Unexpected::Str(&self.value), &"a float"))
            .and_then(|v| visitor.visit_f64(v))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.value)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(&self.value)
    }

    serde::forward_to_deserialize_any! {
        i8 i16 u8 char unit
        bytes byte_buf option unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> IntoDeserializer<'de, de::value::Error> for IniValueDeserializer {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
