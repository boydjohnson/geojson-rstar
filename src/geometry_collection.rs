// Copyright 2020 Boyd Johnson
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

use crate::conversion::create_geo_geometry_collection;
use crate::error::GeoJsonConversionError;
use crate::generic::GenericFeature;
use crate::json::JsonObject;
use crate::{
    LineStringFeature, MultiLineStringFeature, MultiPointFeature, MultiPolygonFeature,
    PointFeature, PolygonFeature,
};
use geo::algorithm::bounding_rect::BoundingRect;
use geo::{Coordinate, Rect};
use geojson::{feature::Id, Bbox};
use geojson::{Geometry, Value};
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryCollectionFeature {
    bbox: Bbox,
    geometries: Vec<Geometry>,
    pub id: Option<Id>,
    pub properties: Option<JsonObject>,
    pub foreign_members: Option<JsonObject>,
}

impl GeometryCollectionFeature {
    pub fn geometries(&self) -> &[Geometry] {
        &self.geometries
    }

    pub fn geo_geometry(&self) -> geo::GeometryCollection<f64> {
        create_geo_geometry_collection(self.geometries())
    }
}

impl Into<geojson::Feature> for GeometryCollectionFeature {
    fn into(self) -> geojson::Feature {
        let geometry = geojson::Geometry::new(geojson::Value::GeometryCollection(self.geometries));

        geojson::Feature {
            id: self.id,
            properties: self.properties,
            foreign_members: self.foreign_members,
            geometry: Some(geometry),
            bbox: Some(self.bbox),
        }
    }
}

impl TryFrom<geojson::Feature> for GeometryCollectionFeature {
    type Error = GeoJsonConversionError;

    fn try_from(feature: geojson::Feature) -> Result<GeometryCollectionFeature, Self::Error> {
        <Self as GenericFeature<GeometryCollectionFeature, Vec<Geometry>>>::try_from(feature)
    }
}

impl GenericFeature<GeometryCollectionFeature, Vec<Geometry>> for GeometryCollectionFeature {
    fn take_geometry_type(
        feature: &mut geojson::Feature,
    ) -> Result<Vec<Geometry>, GeoJsonConversionError> {
        if let geojson::Value::GeometryCollection(geometry_collection) = feature
            .geometry
            .take()
            .ok_or_else(|| {
                let id = feature.id.clone();
                GeoJsonConversionError::MissingGeometry(id)
            })?
            .value
        {
            Ok(geometry_collection)
        } else {
            Err(GeoJsonConversionError::IncorrectGeometryValue(
                "Error: did not find GeometryCollection feature".to_string(),
            ))
        }
    }

    fn check_geometry(
        geometry: &Vec<Geometry>,
        feature: &geojson::Feature,
    ) -> Result<(), GeoJsonConversionError> {
        for geom in geometry {
            let res = match &geom.value {
                Value::Point(p) => PointFeature::check_geometry(&p, feature),
                Value::LineString(l) => LineStringFeature::check_geometry(&l, feature),
                Value::Polygon(p) => PolygonFeature::check_geometry(&p, feature),
                Value::MultiPoint(p) => MultiPointFeature::check_geometry(&p, feature),
                Value::MultiLineString(l) => MultiLineStringFeature::check_geometry(&l, feature),
                Value::MultiPolygon(p) => MultiPolygonFeature::check_geometry(&p, feature),
                Value::GeometryCollection(g) => {
                    GeometryCollectionFeature::check_geometry(&g, feature)
                }
            };
            if res.is_err() {
                return res;
            }
        }
        Ok(())
    }

    fn compute_bbox(feature: &mut geojson::Feature, geometry: &Vec<Geometry>) -> Bbox {
        feature.bbox.take().unwrap_or_else(|| {
            let geo_geometry_collection = create_geo_geometry_collection(geometry);
            let polygons: Vec<geo::Polygon<f64>> = convert_bounding_rect(geo_geometry_collection)
                .into_iter()
                .map(|v| v.into())
                .collect();
            let bounds = geo::MultiPolygon::from(polygons)
                .bounding_rect()
                .expect("Polygons have a bounding rectangle");
            vec![bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y]
        })
    }

    fn create_self(
        feature: geojson::Feature,
        bbox: Bbox,
        geometry: Vec<Geometry>,
    ) -> GeometryCollectionFeature {
        GeometryCollectionFeature {
            bbox,
            id: feature.id,
            geometries: geometry,
            properties: feature.properties,
            foreign_members: feature.foreign_members,
        }
    }
}

fn convert_bounding_rect(geo_geometry_collection: geo::GeometryCollection<f64>) -> Vec<Rect<f64>> {
    geo_geometry_collection
        .into_iter()
        .flat_map(|geo_geom| match geo_geom {
            geo::Geometry::Point(p) => vec![Rect::new(
                Coordinate::from((p.x(), p.y())),
                Coordinate::from((p.x(), p.y())),
            )],
            geo::Geometry::LineString(l) => {
                vec![l.bounding_rect().expect("Expect a bounding rect")]
            }
            geo::Geometry::Polygon(p) => vec![p.bounding_rect().expect("Expect a bounding rect")],
            geo::Geometry::MultiPoint(p) => {
                vec![p.bounding_rect().expect("Expect a bounding rect")]
            }
            geo::Geometry::MultiLineString(l) => {
                vec![l.bounding_rect().expect("Expect a bounding rect")]
            }
            geo::Geometry::MultiPolygon(p) => {
                vec![p.bounding_rect().expect("Expect a bounding rect")]
            }
            geo::Geometry::Line(_) => {
                panic!("GeoJson GeometryCollection Geometry turned into Line, incorrect.");
            }
            geo::Geometry::GeometryCollection(g) => convert_bounding_rect(g),
        })
        .collect()
}
