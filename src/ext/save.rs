use std::path::Path;
use std::fs::File;
use ggez::error::GameError;

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use ::{bincode, Vector2, Point2, GameResult};

/// Save the state in a file
pub fn save<T, P: AsRef<Path>>(path: P, w: &T) -> GameResult<()>
where T: Serialize {
    let mut file = File::create(path)?;
    bincode::serialize_into(&mut file, w, bincode::Infinite)
        .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
    Ok(())
}
/// Load the state from a file
pub fn load<T, P: AsRef<Path>>(path: P, w: &mut T) -> GameResult<()>
where for<'de> T: Deserialize<'de> {
    let mut file = File::open(path)?;
    let save = bincode::deserialize_from(&mut file, bincode::Infinite)
        .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
    *w = save;
    Ok(())
}
/// Serialize a `Point2`
pub fn point_ser<S: Serializer>(p: &Point2, ser: S) -> Result<S::Ok, S::Error> {
    (p.x, p.y).serialize(ser)
}
/// Serialize a `Vector2`
pub fn vec_ser<S: Serializer>(p: &Vector2, ser: S) -> Result<S::Ok, S::Error> {
    (p.x, p.y).serialize(ser)
}
/// Deserialize a `Point2`
pub fn point_des<'de, D: Deserializer<'de>>(des: D) -> Result<Point2, D::Error> {
    <(f32, f32)>::deserialize(des).map(|(x, y)| Point2::new(x, y))
}
/// Deserialize a `Vector2`
pub fn vec_des<'de, D: Deserializer<'de>>(des: D) -> Result<Vector2, D::Error> {
    <(f32, f32)>::deserialize(des).map(|(x, y)| Vector2::new(x, y))
}
