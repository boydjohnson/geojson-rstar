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

//! PolygonFeature can be used with rstar::RTree and carry along the information from the GeoJson

extern crate geojson;

use crate::json::JsonObject;
use geojson::PolygonType;
use geojson::{feature::Id, Bbox};

#[derive(Clone, Debug, PartialEq)]
pub struct PolygonFeature {
    bbox: Bbox,
    polygon: PolygonType,
    pub id: Option<Id>,
    pub properties: Option<JsonObject>,
    pub foreign_members: Option<JsonObject>,
}
