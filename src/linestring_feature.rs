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

//! `LineStringFeature` can be used with rstar `RTree`

use crate::conversion::create_geo_line_string;
use crate::error::GeoJsonConversionError;
use crate::generic::{GenericFeature, GetBbox};
use crate::json::JsonObject;
use geo::algorithm::bounding_rect::BoundingRect;
use geo::algorithm::closest_point::ClosestPoint;
use geo::algorithm::euclidean_length::EuclideanLength;
use geo::algorithm::haversine_distance::HaversineDistance;
use geo::Closest;
use geojson::LineStringType;
use geojson::{feature::Id, Bbox};
use num_traits::identities::Zero;
use rstar::{Envelope, Point, PointDistance, RTreeObject, AABB};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct LineStringFeature {
    bbox: Bbox,
    line: LineStringType,
    pub id: Option<Id>,
    pub properties: Option<JsonObject>,
    pub foreign_members: Option<JsonObject>,
}

impl LineStringFeature {
    pub fn line(&self) -> &LineStringType {
        &self.line
    }

    pub fn geo_line(&self) -> geo::LineString<f64> {
        create_geo_line_string(&self.line)
    }
}

impl Into<geojson::Feature> for LineStringFeature {
    fn into(self) -> geojson::Feature {
        let geometry = geojson::Geometry::new(geojson::Value::LineString(self.line));

        geojson::Feature {
            id: self.id,
            properties: self.properties,
            foreign_members: self.foreign_members,
            geometry: Some(geometry),
            bbox: Some(self.bbox),
        }
    }
}

impl TryFrom<geojson::Feature> for LineStringFeature {
    type Error = GeoJsonConversionError;

    fn try_from(feature: geojson::Feature) -> Result<LineStringFeature, Self::Error> {
        <Self as GenericFeature<LineStringFeature, LineStringType>>::try_from(feature)
    }
}

impl GenericFeature<LineStringFeature, LineStringType> for LineStringFeature {
    fn take_geometry_type(
        feature: &mut geojson::Feature,
    ) -> Result<LineStringType, GeoJsonConversionError> {
        if let geojson::Value::LineString(linestring_type) = feature
            .geometry
            .take()
            .ok_or_else(|| {
                let id = feature.id.clone();
                GeoJsonConversionError::MissingGeometry(id)
            })?
            .value
        {
            Ok(linestring_type)
        } else {
            Err(GeoJsonConversionError::IncorrectGeometryValue(
                "Error: did not find Linestring feature".into(),
            ))
        }
    }

    fn check_geometry(
        geometry: &LineStringType,
        feature: &geojson::Feature,
    ) -> Result<(), GeoJsonConversionError> {
        let euclidean_length: f64 = create_geo_line_string(&geometry).euclidean_length();
        if (euclidean_length - f64::zero()).abs() < ::std::f64::EPSILON {
            let id = feature.id.clone();
            return Err(GeoJsonConversionError::MalformedGeometry(id));
        }
        if geometry.iter().any(|p| p.len() != 2) {
            let id = feature.id.clone();
            return Err(GeoJsonConversionError::MalformedGeometry(id));
        }
        Ok(())
    }

    fn compute_bbox(feature: &mut geojson::Feature, geometry: &LineStringType) -> Bbox {
        feature.bbox.take().unwrap_or_else(|| {
            let geo_linestring = create_geo_line_string(geometry);

            let maybe_rect = geo_linestring
                .bounding_rect()
                .expect("Expect a bounding rect will be produced");
            vec![
                maybe_rect.min().x,
                maybe_rect.min().y,
                maybe_rect.max().x,
                maybe_rect.max().y,
            ]
        })
    }

    fn create_self(
        feature: geojson::Feature,
        bbox: Bbox,
        geometry: LineStringType,
    ) -> LineStringFeature {
        LineStringFeature {
            bbox,
            id: feature.id,
            line: geometry,
            properties: feature.properties,
            foreign_members: feature.foreign_members,
        }
    }
}

impl<'a> GetBbox<'a> for LineStringFeature {
    fn bbox(&'a self) -> &'a Bbox {
        &self.bbox
    }
}

impl RTreeObject for LineStringFeature {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        <Self as GetBbox>::envelope(self)
    }
}

impl PointDistance for LineStringFeature {
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
        let self_linestring = create_geo_line_string(&self.line);

        let geo_point = geo::Point::new(point[0], point[1]);

        let closest = self_linestring.closest_point(&geo_point);
        if let Closest::Intersection(_) = closest {
            0.0
        } else if let Closest::SinglePoint(p) = closest {
            geo_point.haversine_distance(&p)
        } else {
            panic!("The geometry check on LineStringFeature should have ruled this out");
        }
    }
}
