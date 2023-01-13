use std::{fmt::Formatter, str::FromStr};

use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::ResourcePath;

struct PathVisitor {}

impl<'de> Deserialize<'de> for ResourcePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let out = deserializer.deserialize_any(PathVisitor {})?;
        Ok(out)
    }
    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        *place = deserializer.deserialize_any(PathVisitor {})?;
        Ok(())
    }
}

impl<'de> Visitor<'de> for PathVisitor {
    type Value = ResourcePath;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("Except a `ResourcePath` object or a `Url` string.")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match ResourcePath::from_str(v) {
            Ok(o) => Ok(o),
            Err(e) => Err(Error::custom(e.to_string())),
        }
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut network = None;
        let mut relative = None;
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "network" | "remote" => {
                    network = Some(map.next_value()?);
                }
                "relative" | "local" => {
                    relative = Some(map.next_value()?);
                }
                _ => {}
            }
        }
        match (network, relative) {
            (Some(network), Some(relative)) => Ok(ResourcePath { remote: network, local: relative }),
            (_, None) => Err(Error::missing_field("relative")),
            _ => Err(Error::missing_field("network")),
        }
    }
}
