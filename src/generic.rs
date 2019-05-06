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

use crate::error::GeoJsonConversionError;
use geojson::Bbox;

pub(crate) trait GenericFeature<U, G> {
    fn take_geometry_type(feature: &mut geojson::Feature) -> Result<G, GeoJsonConversionError>;

    fn check_geometry(
        geometry: &G,
        feature: &geojson::Feature,
    ) -> Result<(), GeoJsonConversionError>;

    fn compute_bbox(feature: &mut geojson::Feature, geometry: &G) -> Bbox;

    fn create_self(feature: geojson::Feature, bbox: Bbox, geometry: G) -> U;

    fn try_from(mut feature: geojson::Feature) -> Result<U, GeoJsonConversionError> {
        let geometry = Self::take_geometry_type(&mut feature)?;

        Self::check_geometry(&geometry, &feature)?;

        let bbox = feature
            .bbox
            .take()
            .unwrap_or_else(|| Self::compute_bbox(&mut feature, &geometry));

        Ok(Self::create_self(feature, bbox, geometry))
    }
}
