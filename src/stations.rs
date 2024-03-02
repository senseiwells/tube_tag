use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};
use serde::ser::SerializeTuple;

#[derive(Debug, Deserialize)]
pub struct Station {
    pub name: String,
    pub station_offsets: Vec<(f32, f32)>,
    #[serde(default)]
    pub name_data: NameData
}

#[derive(Debug, Default, Deserialize)]
pub struct NameData {
    #[serde(default)]
    pub station_offset: usize,
    #[serde(default)]
    pub anchor: Anchor
}

#[derive(Debug, Default, Deserialize)]
pub enum Anchor {
    #[default]
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest
}