//! Errors that can result from conversion of GeoJson to rstar tree
//! compatible structs.

use geojson::feature::Id;

/// An error that results from failing to convert the GeoJson Feature to
/// a PointFeature, LinestringFeature, PolygonFeature, etc.
#[derive(Debug)]
pub enum GeoJsonConversionError {
    /// The Geometry is missing so no conversion can be made.
    MissingGeometry(Option<Id>),
    /// The Geometry Value variant is wrong for this type.
    IncorrectGeometryValue(String),
    /// The Geometry is malformed, such as a Point has 5 f64s
    MalformedGeometry(Option<Id>),
}
