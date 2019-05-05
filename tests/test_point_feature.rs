extern crate geojson;
extern crate geojson_rstar;
extern crate rstar;
extern crate serde_json;

use geojson::GeoJson;
use geojson_rstar::PointFeature;
use rstar::RTree;
use std::convert::TryInto;

#[test]
fn test_try_into_point_feature_success() {
    let geojson_string = r#"{
        "type": "Feature",
        "properties": { 
            "IPEDSID": "172918",
            "NAME": "ALEXANDRIA TECHNICAL & COMMUNITY COLLEGE",
            "ADDRESS": "1601 JEFFERSON STREEET",
            "CITY": "ALEXANDRIA",
            "STATE": "MN",
            "ZIP": "56308"},
        "geometry": { 
            "type": "Point",
            "coordinates": [ -95.372464158999946, 45.87291786000003 ] } 
        }"#;

    if let GeoJson::Feature(feature) = geojson_string
        .parse::<GeoJson>()
        .expect("The geojson did not correctly parse")
    {
        let point_feature: Result<PointFeature, _> = feature.try_into();
        assert!(
            point_feature.is_ok(),
            "The point geojson feature with a geometry, 
            but missing a bounding box can be converted to a PointFeature"
        )
    } else {
        panic!("The geojson did not parse as a Feature");
    }
}

#[test]
fn test_nearest_neighbor() {
    let points_geojson = r#"{
"type": "FeatureCollection",
"name": "Colleges_Universities",
"crs": { "type": "name", "properties": { "name": "urn:ogc:def:crs:OGC:1.3:CRS84" } },
"bbox": [ -96.096365835, 44.8075502510001, -95.561709068, 46.2873885810001 ],                                                      
"features": [
{ "type": "Feature", "properties": { "IPEDSID": "173559", "NAME": "MINNESOTA STATE COMMUNITY AND TECHNICAL COLLEGE", "ADDRESS": "1414 COLLEGE WAY", "CITY": "FERGUS FALLS", "STATE": "MN", "ZIP": "56537", "ZIP4": "1000", "TELEPHONE": "(218) 736-1500", "TYPE": "1", "STATUS": "A", "POPULATION": 6794, "COUNTY": "OTTER TAIL", "COUNTYFIPS": "27111", "COUNTRY": "USA", "LATITUDE": 46.287388581000073, "LONGITUDE": -96.096365834999972, "NAICS_CODE": "611210", "NAICS_DESC": "JUNIOR COLLEGES", "SOURCE": "https:\/\/nces.ed.gov\/GLOBALLOCATOR\/col_info_popup.asp?ID=173559", "SOURCEDATE": "2009\/10\/13 00:00:00", "VAL_METHOD": "IMAGERY", "VAL_DATE": "2016\/07\/07 00:00:00", "WEBSITE": "www.minnesota.edu", "STFIPS": "27", "COFIPS": "111", "SECTOR": "4", "LEVEL_": "2", "HI_OFFER": "4", "DEG_GRANT": "1", "LOCALE": "33", "CLOSE_DATE": "-2", "MERGE_ID": "-2", "ALIAS": "M STATE", "SIZE_SET": "3", "INST_SIZE": "3", "PT_ENROLL": 3735, "FT_ENROLL": 2581, "TOT_ENROLL": 6316, "HOUSING": "1", "DORM_CAP": 152, "TOT_EMP": 478, "SHELTER_ID": "NOT AVAILABLE" }, "bbox": [ -96.096365834999972, 46.287388581000073, -96.096365834999972, 46.287388581000073 ], "geometry": { "type": "Point", "coordinates": [ -96.096365834999972, 46.287388581000073 ] } },
{ "type": "Feature", "properties": { "IPEDSID": "173638", "NAME": "MINNESOTA WEST COMMUNITY AND TECHNICAL COLLEGE", "ADDRESS": "1593 11TH AVE", "CITY": "GRANITE FALLS", "STATE": "MN", "ZIP": "56241", "ZIP4": "NOT AVAILABLE", "TELEPHONE": "(800) 658-2330", "TYPE": "1", "STATUS": "A", "POPULATION": 3314, "COUNTY": "YELLOW MEDICINE", "COUNTYFIPS": "27173", "COUNTRY": "USA", "LATITUDE": 44.807550251000066, "LONGITUDE": -95.561709067999971, "NAICS_CODE": "611210", "NAICS_DESC": "JUNIOR COLLEGES", "SOURCE": "https:\/\/nces.ed.gov\/GLOBALLOCATOR\/col_info_popup.asp?ID=173638", "SOURCEDATE": "2009\/10\/13 00:00:00", "VAL_METHOD": "IMAGERY", "VAL_DATE": "2016\/07\/07 00:00:00", "WEBSITE": "www.mnwest.edu", "STFIPS": "27", "COFIPS": "173", "SECTOR": "4", "LEVEL_": "2", "HI_OFFER": "4", "DEG_GRANT": "1", "LOCALE": "33", "CLOSE_DATE": "-2", "MERGE_ID": "-2", "ALIAS": "NOT AVAILABLE", "SIZE_SET": "2", "INST_SIZE": "2", "PT_ENROLL": 1956, "FT_ENROLL": 1094, "TOT_ENROLL": 3050, "HOUSING": "1", "DORM_CAP": 16, "TOT_EMP": 264, "SHELTER_ID": "NOT AVAILABLE" }, "bbox": [ -95.561709067999971, 44.807550251000066, -95.561709067999971, 44.807550251000066 ], "geometry": { "type": "Point", "coordinates": [ -95.561709067999971, 44.807550251000066 ] } }
]
}"#;

    let search_point = [-96.47266062208995, 48.465130924829914];

    if let Ok(GeoJson::FeatureCollection(feature_collection)) = points_geojson.parse::<GeoJson>() {
        let feature_points = feature_collection
            .features
            .into_iter()
            .map(|f| f.try_into())
            .collect::<Result<Vec<PointFeature>, _>>()
            .expect("The features were correctly converted as PointFeatures");
        let r_tree = RTree::bulk_load(feature_points);
        let nearest = r_tree
            .nearest_neighbor(&search_point)
            .expect("There is a nearest point in the RTree");

        assert_eq!(
            nearest.properties.as_ref().unwrap().get("IPEDSID"),
            Some(serde_json::Value::String("173559".to_string())).as_ref()
        );
    } else {
        panic!("The geojson did not parse as a FeatureCollection correctly");
    }
}
