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

//! `PointFeature` can be used with `rstar::RTree` and carry along the information from the `GeoJson`

use crate::{
    conversion::create_geo_point,
    error::GeoJsonConversionError,
    generic::{GenericFeature, GetBbox},
    json::JsonObject,
};
use geojson::{feature::Id, Bbox, PointType};
use rstar::{Envelope, Point, PointDistance, RTreeObject, AABB};
use std::convert::TryFrom;

/// `PointFeature` has `TryFrom<geojson::Feature>` and can be used with `RTree`
#[derive(Clone, Debug, PartialEq)]
pub struct PointFeature {
    bbox: Bbox,
    point: PointType,
    pub id: Option<Id>,
    pub properties: Option<JsonObject>,
    pub foreign_members: Option<JsonObject>,
}

impl PointFeature {
    pub fn point(&self) -> &PointType {
        &self.point
    }

    pub fn geo_point(&self) -> geo::Point<f64> {
        create_geo_point(&self.point)
    }
}

impl Into<geojson::Feature> for PointFeature {
    fn into(self) -> geojson::Feature {
        let geometry = geojson::Geometry::new(geojson::Value::Point(self.point));

        geojson::Feature {
            id: self.id,
            properties: self.properties,
            foreign_members: self.foreign_members,
            geometry: Some(geometry),
            bbox: Some(self.bbox),
        }
    }
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

impl<'a> GetBbox<'a> for PointFeature {
    fn bbox(&'a self) -> &'a Bbox {
        &self.bbox
    }
}

impl RTreeObject for PointFeature {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        <Self as GetBbox>::envelope(self)
    }
}

impl PointDistance for PointFeature {
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
        self.geo_point().distance_2(&(*point).into())
    }
}
