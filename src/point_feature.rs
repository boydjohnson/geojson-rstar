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

//! PointFeature can be used with rstar::RTree and carry along the information from the GeoJson

extern crate geo;
extern crate geojson;
extern crate rstar;

use crate::error::GeoJsonConversionError;
use crate::generic::GenericFeature;
use crate::json::JsonObject;
use geo::haversine_distance::HaversineDistance;
use geojson::PointType;
use geojson::{feature::Id, Bbox};
use rstar::{Envelope, Point, PointDistance, RTreeObject, AABB};
use std::convert::TryFrom;

/// PointFeature has TryFrom<geojson::Feature> and can be used with RTree
#[derive(Clone, Debug, PartialEq)]
pub struct PointFeature {
    bbox: Bbox,
    point: PointType,
    pub id: Option<Id>,
    pub properties: Option<JsonObject>,
    pub foreign_members: Option<JsonObject>,
}

impl TryFrom<geojson::Feature> for PointFeature {
    type Error = GeoJsonConversionError;

    fn try_from(feature: geojson::Feature) -> Result<PointFeature, GeoJsonConversionError> {
        <Self as GenericFeature<PointFeature, PointType>>::try_from(feature)
    }
}

impl GenericFeature<PointFeature, PointType> for PointFeature {
    fn take_geometry_type(
        feature: &mut geojson::Feature,
    ) -> Result<PointType, GeoJsonConversionError> {
        if let geojson::Value::Point(point_type) = feature
            .geometry
            .take()
            .ok_or_else(|| {
                let id = feature.id.clone();
                GeoJsonConversionError::MissingGeometry(id)
            })?
            .value
        {
            Ok(point_type)
        } else {
            Err(GeoJsonConversionError::IncorrectGeometryValue(
                "Error: did not find Point feature".into(),
            ))
        }
    }

    fn check_geometry(
        geometry: &PointType,
        feature: &geojson::Feature,
    ) -> Result<(), GeoJsonConversionError> {
        if geometry.len() != 2 {
            let id = feature.id.clone();
            return Err(GeoJsonConversionError::MalformedGeometry(id));
        }
        Ok(())
    }

    fn compute_bbox(feature: &mut geojson::Feature, geometry: &PointType) -> Bbox {
        feature
            .bbox
            .take()
            .unwrap_or_else(|| vec![geometry[0], geometry[1], geometry[0], geometry[1]])
    }

    fn create_self(feature: geojson::Feature, bbox: Bbox, geometry: PointType) -> PointFeature {
        PointFeature {
            bbox,
            id: feature.id,
            point: geometry,
            properties: feature.properties,
            foreign_members: feature.foreign_members,
        }
    }
}

impl RTreeObject for PointFeature {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([
            *self.bbox.get(0).expect("A bounding box has 4 values"),
            *self.bbox.get(1).expect("A bounding box has 4 values"),
        ])
    }
}

impl PointDistance for PointFeature {
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
        let self_point = geo::Point::new(
            *self
                .point
                .get(0)
                .expect("Already checked that PointFeature has 2 values"),
            *self
                .point
                .get(1)
                .expect("Already checked that PointFeature has 2 values"),
        );
        self_point.haversine_distance(&geo::Point::new(point[0], point[1]))
    }
}
