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

//! `MultiPolygonFeature` can be used with `rstar::RTree` and carry along the information from the `GeoJson`

use crate::conversion::create_geo_multi_polygon;
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
pub struct MultiPolygonFeature {
    bbox: Bbox,
    polygons: Vec<PolygonType>,
    pub id: Option<Id>,
    pub properties: Option<JsonObject>,
    pub foreign_members: Option<JsonObject>,
}

impl MultiPolygonFeature {
    pub fn polygons(&self) -> &[PolygonType] {
        &self.polygons
    }
}

impl TryFrom<geojson::Feature> for MultiPolygonFeature {
    type Error = GeoJsonConversionError;

    fn try_from(feature: geojson::Feature) -> Result<Self, Self::Error> {
        <Self as GenericFeature<MultiPolygonFeature, Vec<PolygonType>>>::try_from(feature)
    }
}

impl GenericFeature<MultiPolygonFeature, Vec<PolygonType>> for MultiPolygonFeature {
    fn take_geometry_type(
        feature: &mut geojson::Feature,
    ) -> Result<Vec<PolygonType>, GeoJsonConversionError> {
        if let geojson::Value::MultiPolygon(polygons) = feature
            .geometry
            .take()
            .ok_or_else(|| {
                let id = feature.id.clone();
                GeoJsonConversionError::MissingGeometry(id)
            })?
            .value
        {
            Ok(polygons)
        } else {
            Err(GeoJsonConversionError::IncorrectGeometryValue(
                "Error: did not find a MultiPolygon feature".into(),
            ))
        }
    }

    fn check_geometry(
        geometries: &Vec<PolygonType>,
        feature: &geojson::Feature,
    ) -> Result<(), GeoJsonConversionError> {
        if geometries.iter().any(|g| g.is_empty())
            || geometries.iter().any(|g| g.iter().any(|l| l.is_empty()))
        {
            let id = feature.id.clone();
            return Err(GeoJsonConversionError::MalformedGeometry(id));
        }
        Ok(())
    }

    fn compute_bbox(feature: &mut geojson::Feature, geometries: &Vec<PolygonType>) -> Bbox {
        feature.bbox.take().unwrap_or_else(|| {
            let bounding = create_geo_multi_polygon(geometries)
                .bounding_rect()
                .expect("Geo multipolygon had to bounding rectangle");
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
        geometry: Vec<PolygonType>,
    ) -> MultiPolygonFeature {
        MultiPolygonFeature {
            bbox,
            id: feature.id,
            polygons: geometry,
            properties: feature.properties,
            foreign_members: feature.foreign_members,
        }
    }
}

impl<'a> GetBbox<'a> for MultiPolygonFeature {
    fn bbox(&'a self) -> &'a Bbox {
        &self.bbox
    }
}

impl RTreeObject for MultiPolygonFeature {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        <Self as GetBbox>::envelope(self)
    }
}

impl PointDistance for MultiPolygonFeature {
    fn distance_2(
        &self,
        point: &<Self::Envelope as Envelope>::Point,
    ) -> <<Self::Envelope as Envelope>::Point as Point>::Scalar {
        let self_m_polygon = create_geo_multi_polygon(&self.polygons);

        let geo_point = geo::Point::new(point[0], point[1]);

        let closest = self_m_polygon.closest_point(&geo_point);

        if let Closest::Intersection(_) = closest {
            0.0
        } else if let Closest::SinglePoint(p) = closest {
            geo_point.haversine_distance(&p)
        } else {
            panic!("MultiPolygon Closest point will not be indeterminate")
        }
    }
}
