extern crate geojson;
extern crate geojson_rstar;
extern crate rstar;
extern crate serde_json;

use geojson::GeoJson;
use geojson_rstar::LineStringFeature;
use rstar::RTree;
use std::convert::TryInto;

#[test]
fn test_try_into_linestring_feature_success() {
    let json_string = r#"{ "type": "Feature", "properties": { "STREETALL": "GLEASON LAKE DR", "ALT_NAM1": " ", "ALT_NAM2": " ", "TLGID": 43062, "FUN_CLAS": 310, "FC_NAME": "Major Collector", "GlobalID": "{0B911CDA-9962-4D2C-8FBF-BF79DD616DEC}", "Shape_Length": 221.56369527898175 }, "geometry": { "type": "LineString", "coordinates": [ [ 462822.7187, 4980845.5 ], [ 462866.6875, 4980852.0 ], [ 463035.5312, 4980905.5 ] ] } }"#;
    let linestring_geojson = json_string
        .parse::<GeoJson>()
        .expect("Expect that the geojson parsed");

    if let GeoJson::Feature(feature) = linestring_geojson {
        let linestring_feature: Result<LineStringFeature, _> = feature.try_into();

        assert!(
            linestring_feature.is_ok(),
            "Can convert into LineStringFeature from LineString"
        );
    } else {
        panic!("Linestring geojson did not parse into a feature")
    }
}

#[test]
fn test_nearest_neighbor() {
    let linestring_geojson = r#"{
"type": "FeatureCollection",
"name": "FunctionalClassRoads",
"crs": { "type": "name", "properties": { "name": "urn:ogc:def:crs:EPSG::26915" } },
"features": [
{ "type": "Feature", "properties": { "STREETALL": "280TH ST W", "ALT_NAM1": "HIGHWAY 19", "ALT_NAM2": " ", "TLGID": 382980, "FUN_CLAS": 214, "FC_NAME": "A Minor Connector", "GlobalID": "{3BFB4F09-0E4E-4544-BBF6-9CA23CC552F2}", "Shape_Length": 648.70834892673645 }, "geometry": { "type": "LineString", "coordinates": [ [ 452561.818099999800324, 4932439.987099999562 ], [ 452352.024799999780953, 4932443.138499999419 ], [ 451913.179999999701977, 4932449.5327 ] ] } },
{ "type": "Feature", "properties": { "STREETALL": "BAVARIA RD", "ALT_NAM1": " ", "ALT_NAM2": " ", "TLGID": 232838, "FUN_CLAS": 310, "FC_NAME": "Major Collector", "GlobalID": "{47997D49-A623-4B53-8EA0-D78814C5C3E0}", "Shape_Length": 446.19254281689319 }, "geometry": { "type": "LineString", "coordinates": [ [ 451135.8903, 4964392.923699999228 ], [ 451127.961400000378489, 4964405.121899999678 ], [ 451122.472199999727309, 4964415.185399999842 ], [ 451105.307500000111759, 4964448.53559999913 ], [ 451088.340300000272691, 4964473.8761 ], [ 451056.694199999794364, 4964519.031199999154 ], [ 451029.605399999767542, 4964551.5645 ], [ 450988.458999999798834, 4964590.5532 ], [ 450911.251899999566376, 4964658.255 ], [ 450869.971699999645352, 4964689.215099999681 ], [ 450830.067599999718368, 4964707.103199999779 ] ] } }
]
}
"#;

    let json = linestring_geojson
        .parse::<GeoJson>()
        .expect("Expect that the geojson parsed");

    let search_point = [452561.8180999798, 4932439.98709969];

    if let GeoJson::FeatureCollection(collection) = json {
        let linestring_features = collection
            .features
            .into_iter()
            .map(|l| l.try_into())
            .collect::<Result<Vec<LineStringFeature>, _>>();
        assert!(
            linestring_features.is_ok(),
            "The linestrings can be converted to linestring features"
        );
        let lsf = linestring_features.expect("The linestring features were correctly converted");
        let tree = RTree::bulk_load(lsf);

        let nearest = tree
            .nearest_neighbor(&search_point)
            .expect("There is a nearest neighbor");
        assert_eq!(
            nearest.properties.as_ref().unwrap().get("STREETALL"),
            Some(&serde_json::Value::String("280TH ST W".into())),
            "The closest linestring is correctly found"
        );
    }
}
