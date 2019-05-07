extern crate geojson;
extern crate geojson_rstar;
extern crate rstar;
extern crate serde_json;

use geojson::GeoJson;
use geojson_rstar::PolygonFeature;
use rstar::RTree;
use std::convert::TryInto;

#[test]
fn test_try_from_polygon_success() {
    let geojson_string = r#"{ "type": "Feature", "properties": { "STATEFP": "26", "COUNTYFP": "035", "COUNTYNS": "01622960", "GEOID": "26035", "NAME": "Clare", "NAMELSAD": "Clare County", "LSAD": "06", "CLASSFP": "H1", "MTFCC": "G4020", "CSAFP": null, "CBSAFP": null, "METDIVFP": null, "FUNCSTAT": "A", "ALAND": 1461700230, "AWATER": 28500463, "INTPTLAT": "+43.9911368", "INTPTLON": "-084.8383253" }, "geometry": { "type": "Polygon", "coordinates": [ [ [ -85.087602, 44.07366 ], [ -84.608104, 44.160482 ], [ -84.606037, 43.815365 ], [ -85.088811, 43.813676 ], [ -85.087602, 44.07366 ] ] ] } }"#;

    if let GeoJson::Feature(feature) = geojson_string
        .parse::<GeoJson>()
        .expect("The geojson did not correctly parse")
    {
        let polygon_feature: Result<PolygonFeature, _> = feature.try_into();
        assert!(
            polygon_feature.is_ok(),
            "The polygon feature can be correctly converted from polygon geojson feature"
        )
    }
}

#[test]
fn test_nearest_neighbor() {
    let polygon_geojson = r#"{
"type": "FeatureCollection",
"name": "tl_2018_us_county",
"crs": { "type": "name", "properties": { "name": "urn:ogc:def:crs:EPSG::4269" } },
"features": [
{ "type": "Feature", "properties": { "STATEFP": "53", "COUNTYFP": "069", "COUNTYNS": "01513275", "GEOID": "53069", "NAME": "Wahkiakum", "NAMELSAD": "Wahkiakum County", "LSAD": "06", "CLASSFP": "H1", "MTFCC": "G4020", "CSAFP": null, "CBSAFP": null, "METDIVFP": null, "FUNCSTAT": "A", "ALAND": 680956809, "AWATER": 61588406, "INTPTLAT": "+46.2946377", "INTPTLON": "-123.4244583" }, "geometry": { "type": "Polygon", "coordinates": [ [ [ -123.436394, 46.238197 ], [ -123.728316, 46.264541 ], [ -123.726557, 46.384872 ], [ -123.21795, 46.385617 ], [ -123.436394, 46.238197 ] ] ] } },
{ "type": "Feature", "properties": { "STATEFP": "26", "COUNTYFP": "035", "COUNTYNS": "01622960", "GEOID": "26035", "NAME": "Clare", "NAMELSAD": "Clare County", "LSAD": "06", "CLASSFP": "H1", "MTFCC": "G4020", "CSAFP": null, "CBSAFP": null, "METDIVFP": null, "FUNCSTAT": "A", "ALAND": 1461700230, "AWATER": 28500463, "INTPTLAT": "+43.9911368", "INTPTLON": "-084.8383253" }, "geometry": { "type": "Polygon", "coordinates": [ [ [ -85.087602, 44.07366 ], [ -84.608104, 44.160482 ], [ -84.606037, 43.815365 ], [ -85.088811, 43.813676 ], [ -85.087602, 44.07366 ] ] ] } }
]
}
"#;

    let search_point = [-118.0, 34.0];

    if let Ok(GeoJson::FeatureCollection(collection)) = polygon_geojson.parse::<GeoJson>() {
        let feature_polygons = collection
            .features
            .into_iter()
            .map(|f| f.try_into())
            .collect::<Result<Vec<PolygonFeature>, _>>()
            .expect("The features were correctly converted");

        let tree = RTree::bulk_load(feature_polygons);

        let nearest = tree
            .nearest_neighbor(&search_point)
            .expect("There is a nearest polygon");
        assert_eq!(
            nearest.properties.as_ref().unwrap().get("STATEFP"),
            Some(&serde_json::Value::String("53".into()))
        );
    } else {
        panic!("The geojson did not parse correctly");
    }
}
