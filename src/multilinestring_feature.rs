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

//! `MultiLineStringFeature` can be used with rstar `RTree`

use crate::conversion::create_geo_multi_line_string;
use crate::error::GeoJsonConversionError;
use crate::generic::{GenericFeature, GetBbox};
use crate::json::JsonObject;
use geo::bounding_rect::BoundingRect;
use geo::haversine_distance::HaversineDistance;
use geo::{closest_point::ClosestPoint, Closest};
use geojson::LineStringType;
use geojson::{feature::Id, Bbox};
use rstar::{Envelope, Point, PointDistance, RTreeObject, AABB};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct MultiLineStringFeature {
    bbox: Bbox,
    lines: Vec<LineStringType>,
    pub id: Option<Id>,
    pub properties: Option<JsonObject>,
    pub foreign_members: Option<JsonObject>,
}

impl MultiLineStringFeature {
    pub fn lines(&self) -> &[LineStringType] {
        &self.lines
    }
}

impl TryFrom<geojson::Feature> for MultiLineStringFeature {
    type Error = GeoJsonConversionError;

    fn try_from(
        feature: geojson::Feature,
    ) -> Result<MultiLineStringFeature, GeoJsonConversionError> {
        <Self as GenericFeature<MultiLineStringFeature, Vec<LineStringType>>>::try_from(feature)
    }
}

impl GenericFeature<MultiLineStringFeature, Vec<LineStringType>> for MultiLineStringFeature {
    fn take_geometry_type(
        feature: &mut geojson::Feature,
    ) -> Result<Vec<LineStringType>, GeoJsonConversionError> {
        if let geojson::Value::MultiLineString(lines) = feature
            .geometry
            .take()
            .ok_or_else(|| {
                let id = feature.id.clone();
                GeoJsonConversionError::MissingGeometry(id)
            })?
            .value
        {
            Ok(lines)
        } else {
            Err(GeoJsonConversionError::IncorrectGeometryValue(
                "Error: did not find a MultiLinestring Feature".into(),
            ))
        }
    }

    fn check_geometry(
        geometry: &Vec<LineStringType>,
        feature: &geojson::Feature,
    ) -> Result<(), GeoJsonConversionError> {
        if geometry.is_empty() || geometry.iter().any(|l| l.is_empty()) {
            let id = feature.id.clone();
            return Err(GeoJsonConversionError::MalformedGeometry(id));
        }
        Ok(())
    }

    fn compute_bbox(feature: &mut geojson::Feature, geometry: &Vec<LineStringType>) -> Bbox {
        feature.bbox.take().unwrap_or_else(|| {
            let maybe_rect = create_geo_multi_line_string(geometry)
                .bounding_rect()
                .expect("Was able to create bounding rectangle");
            vec![
                maybe_rect.min.x,
                maybe_rect.min.y,
                maybe_rect.max.x,
                maybe_rect.max.y,
            ]
        })
    }

    fn create_self(
        feature: geojson::Feature,
        bbox: Bbox,
        geometry: Vec<LineStringType>,
    ) -> MultiLineStringFeature {
        MultiLineStringFeature {
            id: feature.id,
            bbox,
            lines: geometry,
            properties: feature.properties,
            foreign_members: feature.foreign_members,
        }
    }
}

impl<'a> GetBbox<'a> for MultiLineStringFeature {
    fn bbox(&'a self) -> &'a Bbox {
        &self.bbox
    }
}

impl RTreeObject for MultiLineStringFeature {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        <Self as GetBbox>::envelope(self)
    }
}

impl PointDistance for MultiLineStringFeature {
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
        let self_lines = create_geo_multi_line_string(&self.lines);

        let geo_point = geo::Point::new(point[0], point[1]);

        let closest = self_lines.closest_point(&geo_point);
        if let Closest::Intersection(_) = closest {
            0.0
        } else if let Closest::SinglePoint(p) = closest {
            p.haversine_distance(&geo_point)
        } else {
            unimplemented!()
        }
    }
}
