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

use crate::conversion::create_geo_polygon;
use crate::error::GeoJsonConversionError;
use crate::generic::{GenericFeature, GetBbox};
use crate::json::JsonObject;
use geo::bounding_rect::BoundingRect;
use geo::closest_point::ClosestPoint;
use geo::haversine_distance::HaversineDistance;
use geo::Closest;
use geojson::PolygonType;
use geojson::{feature::Id, Bbox};
use rstar::{Envelope, Point, PointDistance, RTreeObject, AABB};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct PolygonFeature {
    bbox: Bbox,
    polygon: PolygonType,
    pub id: Option<Id>,
    pub properties: Option<JsonObject>,
    pub foreign_members: Option<JsonObject>,
}

impl PolygonFeature {
    pub fn polygon(&self) -> &PolygonType {
        &self.polygon
    }
}

impl TryFrom<geojson::Feature> for PolygonFeature {
    type Error = GeoJsonConversionError;

    fn try_from(feature: geojson::Feature) -> Result<Self, Self::Error> {
        <Self as GenericFeature<PolygonFeature, PolygonType>>::try_from(feature)
    }
}

impl GenericFeature<PolygonFeature, PolygonType> for PolygonFeature {
    fn take_geometry_type(
        feature: &mut geojson::Feature,
    ) -> Result<PolygonType, GeoJsonConversionError> {
        if let geojson::Value::Polygon(polygon) = feature
            .geometry
            .take()
            .ok_or_else(|| {
                let id = feature.id.clone();
                GeoJsonConversionError::MissingGeometry(id)
            })?
            .value
        {
            Ok(polygon)
        } else {
            Err(GeoJsonConversionError::IncorrectGeometryValue(
                "Error: did not find Polygon feature".into(),
            ))
        }
    }

    fn check_geometry(
        geometry: &PolygonType,
        feature: &geojson::Feature,
    ) -> Result<(), GeoJsonConversionError> {
        if geometry
            .iter()
            .any(|line| line.iter().any(|p| p.len() != 2))
        {
            let id = feature.id.clone();
            return Err(GeoJsonConversionError::MalformedGeometry(id));
        }

        if geometry.is_empty() || geometry.iter().any(|v| v.is_empty()) {
            let id = feature.id.clone();
            return Err(GeoJsonConversionError::MalformedGeometry(id));
        }
        Ok(())
    }

    fn compute_bbox(feature: &mut geojson::Feature, geometry: &PolygonType) -> Bbox {
        feature.bbox.take().unwrap_or_else(|| {
            let maybe_rect = create_geo_polygon(geometry)
                .bounding_rect()
                .expect("Expect a bounding rectangle");
            vec![
                maybe_rect.min.x,
                maybe_rect.min.y,
                maybe_rect.max.x,
                maybe_rect.max.y,
            ]
        })
    }

    fn create_self(feature: geojson::Feature, bbox: Bbox, geometry: PolygonType) -> PolygonFeature {
        PolygonFeature {
            bbox,
            id: feature.id,
            polygon: geometry,
            properties: feature.properties,
            foreign_members: feature.foreign_members,
        }
    }
}

impl<'a> GetBbox<'a> for PolygonFeature {
    fn bbox(&'a self) -> &'a Bbox {
        &self.bbox
    }
}

impl RTreeObject for PolygonFeature {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        <Self as GetBbox>::envelope(self)
    }
}

impl PointDistance for PolygonFeature {
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
        let self_polygon = create_geo_polygon(&self.polygon);

        let geo_point = geo::Point::new(point[0], point[1]);

        let closest = self_polygon.closest_point(&geo_point);
        if let Closest::Intersection(_) = closest {
            0.0
        } else if let Closest::SinglePoint(p) = closest {
            geo_point.haversine_distance(&p)
        } else {
            panic!("Polygon Closest point will not be indeterminate");
        }
    }
}
