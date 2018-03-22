use serde::{Serialize, Deserialize, Serializer, Deserializer};
use ::{Vector2, Point2};

/// Serialize a `Point2`
pub fn point_ser<S: Serializer>(p: &Point2, ser: S) -> Result<S::Ok, S::Error> {
    (p.x, p.y).serialize(ser)
}
/// Serialize a `Vector2`
#[allow(dead_code)]
pub fn vec_ser<S: Serializer>(p: &Vector2, ser: S) -> Result<S::Ok, S::Error> {
    (p.x, p.y).serialize(ser)
}
/// Deserialize a `Point2`
pub fn point_des<'de, D: Deserializer<'de>>(des: D) -> Result<Point2, D::Error> {
    <(f32, f32)>::deserialize(des).map(|(x, y)| Point2::new(x, y))
}
/// Deserialize a `Vector2`
#[allow(dead_code)]
pub fn vec_des<'de, D: Deserializer<'de>>(des: D) -> Result<Vector2, D::Error> {
    <(f32, f32)>::deserialize(des).map(|(x, y)| Vector2::new(x, y))
}
