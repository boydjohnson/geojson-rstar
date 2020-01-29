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

//! `MultiPointFeature` can be used with `rstar::RTree` and carry along the information from the `GeoJson`

use crate::conversion::create_geo_multi_point;
use crate::error::GeoJsonConversionError;
use crate::generic::{GenericFeature, GetBbox};
use crate::json::JsonObject;
use geo::algorithm::closest_point::ClosestPoint;
use geo::bounding_rect::BoundingRect;
use geo::haversine_distance::HaversineDistance;
use geo::Closest;
use geojson::PointType;
use geojson::{feature::Id, Bbox};
use rstar::{Envelope, Point, PointDistance, RTreeObject, AABB};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct MultiPointFeature {
    bbox: Bbox,
    points: Vec<PointType>,
    pub id: Option<Id>,
    pub properties: Option<JsonObject>,
    pub foreign_members: Option<JsonObject>,
}

impl MultiPointFeature {
    pub fn points(&self) -> &[PointType] {
        &self.points
    }

    pub fn geo_points(&self) -> geo::MultiPoint<f64> {
        create_geo_multi_point(&self.points)
    }
}

impl Into<geojson::Feature> for MultiPointFeature {
    fn into(self) -> geojson::Feature {
        let geometry = geojson::Geometry::new(geojson::Value::MultiPoint(self.points));

        geojson::Feature {
            id: self.id,
            properties: self.properties,
            foreign_members: self.foreign_members,
            geometry: Some(geometry),
            bbox: Some(self.bbox),
        }
    }
}

impl TryFrom<geojson::Feature> for MultiPointFeature {
    type Error = GeoJsonConversionError;

    fn try_from(feature: geojson::Feature) -> Result<MultiPointFeature, GeoJsonConversionError> {
        <Self as GenericFeature<MultiPointFeature, Vec<PointType>>>::try_from(feature)
    }
}

impl GenericFeature<MultiPointFeature, Vec<PointType>> for MultiPointFeature {
    fn take_geometry_type(
        feature: &mut geojson::Feature,
    ) -> Result<Vec<PointType>, GeoJsonConversionError> {
        if let geojson::Value::MultiPoint(points) = feature
            .geometry
            .take()
            .ok_or_else(|| {
                let id = feature.id.clone();
                GeoJsonConversionError::MissingGeometry(id)
            })?
            .value
        {
            Ok(points)
        } else {
            Err(GeoJsonConversionError::IncorrectGeometryValue(
                "Error: did not find a MultiPoint feature".into(),
            ))
        }
    }

    fn check_geometry(
        geometry: &Vec<PointType>,
        feature: &geojson::Feature,
    ) -> Result<(), GeoJsonConversionError> {
        if geometry.is_empty() || geometry.iter().any(|g| g.len() != 2) {
            let id = feature.id.clone();
            return Err(GeoJsonConversionError::MalformedGeometry(id));
        }
        Ok(())
    }

    fn compute_bbox(feature: &mut geojson::Feature, geometry: &Vec<PointType>) -> Bbox {
        feature.bbox.take().unwrap_or_else(|| {
            let bounding = create_geo_multi_point(geometry)
                .bounding_rect()
                .expect("Geo multi point had no bounding rectangle");
            vec![
                bounding.min.x,
                bounding.min.y,
                bounding.max.x,
                bounding.max.y,
            ]
        })
    }

    fn create_self(
        feature: geojson::Feature,
        bbox: Bbox,
        geometry: Vec<PointType>,
    ) -> MultiPointFeature {
        MultiPointFeature {
            bbox,
            id: feature.id,
            points: geometry,
            properties: feature.properties,
            foreign_members: feature.foreign_members,
        }
    }
}

impl<'a> GetBbox<'a> for MultiPointFeature {
    fn bbox(&'a self) -> &'a Bbox {
        &self.bbox
    }
}

impl RTreeObject for MultiPointFeature {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        <Self as GetBbox>::envelope(self)
    }
}

impl PointDistance for MultiPointFeature {
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
        let self_points = create_geo_multi_point(&self.points);

        let geo_point = geo::Point::new(point[0], point[1]);

        let closest = self_points.closest_point(&geo_point);
        if let Closest::Intersection(_) = closest {
            0.0
        } else if let Closest::SinglePoint(p) = closest {
            geo_point.haversine_distance(&p)
        } else {
            unimplemented!()
        }
    }
}
