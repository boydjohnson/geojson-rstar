extern crate geojson;
extern crate rstar;

pub mod error;
pub mod point_feature;

pub use point_feature::PointFeature;

mod json {
    use serde_json::{Map, Value as JsonValue};
    pub type JsonObject = Map<String, JsonValue>;
}
