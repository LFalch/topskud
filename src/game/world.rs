use ::*;

use std::path::Path;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use ggez::error::GameError;

use ::bincode;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug, Serialize, Deserialize)]
/// All the objects in the current world
pub struct World {
    pub(super) player: Object,
    pub(super) grid: Grid,
    pub(super) bullets: Vec<Object>,
    pub(super) holes: Vec<Object>,
}

include!("material_macro.rs");

mat!{
    MISSING = Missing
    Grass = 0, Grass, false,
    Wall = 1, Wall, true,
    Floor = 2, Floor, false,
    Dirt = 3, Dirt, false,
    Missing = 404, Missing, true,
}

#[derive(Debug, Clone)]
pub struct Level {
    pub grid: Grid,
    pub start_point: Option<Point2>,
}

impl Level {
    pub fn new() -> Self {
        Level {
            grid: Grid::new(),
            start_point: None,
        }
    }
    pub fn load<P: AsRef<Path>>(path: P) -> GameResult<Self> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut ret = Level::new();
        ret.grid = bincode::deserialize_from(&mut reader, bincode::Infinite)
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;

        loop {
            let mut buf = String::with_capacity(16);
            reader.read_line(&mut buf)?;
            if buf.is_empty() {
                break
            }
            if buf == "\n" {
                continue
            }
            match &*buf.trim_right() {
                "START" => ret.start_point = Some(
                    bincode::deserialize_from(&mut reader, bincode::Infinite)
                        .map(|(x, y)| Point2::new(x, y))
                        .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?
                ),
                "END" => break,
                _ => return Err("Bad section".to_string())?
            }
        }

        Ok(ret)
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) -> GameResult<()> {
        let mut file = File::create(path)?;
        bincode::serialize_into(&mut file, &self.grid, bincode::Infinite)
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        if let Some(start) = self.start_point {
            writeln!(file, "\nSTART")?;
            bincode::serialize_into(&mut file, &(start.x, start.y), bincode::Infinite)
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }

        writeln!(file, "\nEND")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid([[Material; 32]; 32]);

impl Grid {
    pub fn new() -> Self {
        Grid([[Material::Grass; 32]; 32])
    }
    #[inline]
    pub fn snap(c: Point2) -> (usize, usize) {
        Self::snap_coords(c.x, c.y)
    }
    pub fn snap_coords(x: f32, y: f32) -> (usize, usize) {
        let x = ((x) / 32.) as usize;
        let y = ((y) / 32.) as usize;

        (x, y)
    }
    pub fn get(&self, x: usize, y: usize) -> Material {
        self.0[x][y]
    }
    pub fn insert(&mut self, x: usize, y: usize, mat: Material) {
        self.0[x][y] = mat;
    }
    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        for (i, vert) in self.0.iter().enumerate() {
            for (j, mat) in vert.iter().enumerate() {
                let x = i as f32 * 32.;
                let y = j as f32 * 32.;

                mat.draw(ctx, assets, x, y)?;
            }
        }
        Ok(())
    }
}
