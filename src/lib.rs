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

pub mod conversion;
pub mod error;
pub mod generic;
pub mod linestring_feature;
pub mod multilinestring_feature;
pub mod multipoint_feature;
pub mod multipolygon_feature;
pub mod point_feature;
pub mod polygon_feature;

pub use error::GeoJsonConversionError;
pub use linestring_feature::LineStringFeature;
pub use multilinestring_feature::MultiLineStringFeature;
pub use multipoint_feature::MultiPointFeature;
pub use multipolygon_feature::MultiPolygonFeature;
pub use point_feature::PointFeature;
pub use polygon_feature::PolygonFeature;

use std::convert::TryFrom;

mod json {
    use serde_json::{Map, Value as JsonValue};
    pub type JsonObject = Map<String, JsonValue>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Feature {
    Point(PointFeature),
    Polygon(PolygonFeature),
    LineString(LineStringFeature),
    MultiPoint(MultiPointFeature),
    MultiLineString(MultiLineStringFeature),
    MultiPolygon(MultiPolygonFeature),
}

impl rstar::RTreeObject for Feature {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        match self {
            Feature::Point(point) => point.envelope(),
            Feature::Polygon(polygon) => polygon.envelope(),
            Feature::LineString(line) => line.envelope(),
            Feature::MultiPoint(mpoint) => mpoint.envelope(),
            Feature::MultiLineString(mline) => mline.envelope(),
            Feature::MultiPolygon(mpolygon) => mpolygon.envelope(),
        }
    }
}

impl rstar::PointDistance for Feature {
    fn distance_2(
        &self,
        point: &<Self::Envelope as rstar::Envelope>::Point,
    ) -> <<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar {
        match self {
            Feature::Point(p) => p.distance_2(point),
            Feature::Polygon(polygon) => polygon.distance_2(point),
            Feature::LineString(line) => line.distance_2(point),
            Feature::MultiPoint(mpoint) => mpoint.distance_2(point),
            Feature::MultiLineString(mline) => mline.distance_2(point),
            Feature::MultiPolygon(mpolygon) => mpolygon.distance_2(point),
        }
    }
}

impl TryFrom<geojson::Feature> for Feature {
    type Error = GeoJsonConversionError;

    fn try_from(feature: geojson::Feature) -> Result<Feature, Self::Error> {
        match feature.geometry.as_ref().map(|g| &g.value) {
            Some(geojson::Value::Point(_)) => PointFeature::try_from(feature).map(Feature::Point),
            Some(geojson::Value::LineString(_)) => {
                LineStringFeature::try_from(feature).map(Feature::LineString)
            }
            Some(geojson::Value::Polygon(_)) => {
                PolygonFeature::try_from(feature).map(Feature::Polygon)
            }
            Some(geojson::Value::MultiPoint(_)) => {
                MultiPointFeature::try_from(feature).map(Feature::MultiPoint)
            }
            Some(geojson::Value::MultiLineString(_)) => {
                MultiLineStringFeature::try_from(feature).map(Feature::MultiLineString)
            }
            Some(geojson::Value::MultiPolygon(_)) => {
                MultiPolygonFeature::try_from(feature).map(Feature::MultiPolygon)
            }
            Some(geojson::Value::GeometryCollection(_)) => {
                panic!("GeometryCollection is not implemented yet")
            }
            None => Err(GeoJsonConversionError::MissingGeometry(feature.id)),
        }
    }
}

impl Into<geojson::Feature> for Feature {
    fn into(self) -> geojson::Feature {
        match self {
            Feature::Point(p) => p.into(),
            Feature::LineString(l) => l.into(),
            Feature::Polygon(p) => p.into(),
            Feature::MultiPoint(p) => p.into(),
            Feature::MultiLineString(l) => l.into(),
            Feature::MultiPolygon(p) => p.into(),
        }
    }
}
