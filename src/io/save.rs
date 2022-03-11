use serde::{Serialize, Deserialize, Serializer, Deserializer};
use crate::util::{Vector2, Point2};

/// Serialize a `Vector2`
#[allow(dead_code)]
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn vec_ser<S: Serializer>(p: &Vector2, ser: S) -> Result<S::Ok, S::Error> {
    (p.x, p.y).serialize(ser)
}
/// Deserialize a `Vector2`
#[allow(dead_code)]
pub fn vec_des<'de, D: Deserializer<'de>>(des: D) -> Result<Vector2, D::Error> {
    <(f32, f32)>::deserialize(des).map(|(x, y)| vector!(x, y))
}

#[inline]
#[allow(clippy::trivially_copy_pass_by_ref, dead_code)]
fn p(p: &Point2) -> (f32, f32) {
    (p.x, p.y)
}
// #[inline]
// fn v(v: &Vector2) -> (f32, f32) {
//     (v.x, v.y)
// }

#[derive(Serialize, Deserialize)]
#[serde(remote = "Point2")]
pub struct Point2Def {
    #[serde(getter = "p")]
    coords: (f32, f32)
}

impl From<Point2Def> for Point2 {
    fn from(def: Point2Def) -> Self {
        point!(def.coords.0, def.coords.1)
    }
}

// #[derive(Serialize, Deserialize)]
// #[serde(remote = "Vector2")]
// struct Vector2Def {
//     #[serde(getter = "v")]
//     coords: (f32, f32)
// }

// impl From<Vector2Def> for Vector2 {
//     fn from(def: Vector2Def) -> Self {
//         vector!(def.coords.0, def.coords.1)
//     }
// }