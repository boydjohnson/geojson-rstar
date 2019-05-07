// Copyright 2019 Boyd Johnson
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate geojson;
extern crate rstar;

pub mod conversion;
pub mod error;
pub mod generic;
pub mod linestring_feature;
pub mod multilinestring_feature;
pub mod multipoint_feature;
pub mod multipolygon_feature;
pub mod point_feature;
pub mod polygon_feature;

pub use point_feature::PointFeature;

mod json {
    use serde_json::{Map, Value as JsonValue};
    pub type JsonObject = Map<String, JsonValue>;
}
