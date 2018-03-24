use ::*;

use std::path::Path;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use ggez::error::GameError;

use ::bincode;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug)]
/// All the objects in the current world
pub struct World {
    pub(super) player: Object,
    pub(super) grid: Grid,
    pub(super) exit: Option<Point2>,
    pub(super) intels: Vec<Point2>,
    pub(super) enemies: Vec<Enemy>,
    pub(super) bullets: Vec<Object>,
}

include!("material_macro.rs");

mat!{
    MISSING = Missing
    Grass = 0, Grass, false,
    Wall = 1, Wall, true,
    Floor = 2, Floor, false,
    Dirt = 3, Dirt, false,
    Asphalt = 4, Asphalt, false,
    Sand = 5, Sand, false,
    Concrete = 6, Concrete, true,
    Missing = 404, Missing, true,
}

#[derive(Debug, Clone)]
pub struct Level {
    pub grid: Grid,
    pub start_point: Option<Point2>,
    pub enemies: Vec<Enemy>,
    pub exit: Option<Point2>,
    pub intels: Vec<Point2>,
}

impl Level {
    pub fn new(width: usize, height: usize) -> Self {
        Level {
            grid: Grid::new(width, height),
            start_point: None,
            enemies: Vec::new(),
            exit: None,
            intels: Vec::new(),
        }
    }
    pub fn load<P: AsRef<Path>>(path: P) -> GameResult<Self> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut ret = Level::new(0, 0);

        loop {
            let mut buf = String::with_capacity(16);
            reader.read_line(&mut buf)?;
            if buf == "\n" {
                continue
            }
            match &*buf.trim_right() {
                "GRID" => ret.grid = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?,
                "START" => ret.start_point = Some(
                    bincode::deserialize_from(&mut reader)
                        .map(|(x, y)| Point2::new(x, y))
                        .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?
                ),
                "ENEMIES" => ret.enemies = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?,
                "POINT GOAL" => ret.exit = Some(bincode::deserialize_from(&mut reader)
                    .map(|(x, y)| Point2::new(x, y))
                    .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?),
                "INTELS" => ret.intels = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<(f32, f32)>| l.into_iter().map(|(x, y)| Point2::new(x, y)).collect())
                    .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?,
                "END" => break,
                _ => return Err("Bad section".to_string())?
            }
        }

        Ok(ret)
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) -> GameResult<()> {
        let mut file = File::create(path)?;

        writeln!(file, "GRID")?;
        bincode::serialize_into(&mut file, &self.grid)
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        if let Some(start) = self.start_point {
            writeln!(file, "\nSTART")?;
            bincode::serialize_into(&mut file, &(start.x, start.y))
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }
        if !self.enemies.is_empty() {
            writeln!(file, "\nENEMIES")?;
            bincode::serialize_into(&mut file, &self.enemies)
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }
        if let Some(p) = self.exit {
            writeln!(file, "\nPOINT GOAL")?;
            bincode::serialize_into(&mut file, &(p.x, p.y))
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }
        if !self.intels.is_empty() {
            writeln!(file, "\nINTELS")?;
            let intels: Vec<_> = self.intels.iter().map(|p| (p.x, p.y)).collect();
            bincode::serialize_into(&mut file, &intels)
                .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }

        writeln!(file, "\nEND")?;
        Ok(())
    }
    pub fn from_32x32_transposed_grid(mats: [[Material; 32]; 32]) -> Self {
        let mut vec = Vec::with_capacity(1024);
        for y in 0..32 {
            for x in 0..32 {
                vec.push(mats[x][y]);
            }
        }

        let grid = Grid {
            width: 32,
            mats: vec,
        };

        Level {
            grid,
            intels: Vec::new(),
            exit: None,
            start_point: None,
            enemies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid{
    width: usize,
    mats: Vec<Material>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Grid {
            width: width,
            mats: vec![Material::Grass; width*height],
        }
    }
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.mats.len() / self.width
    }
    #[inline]
    pub fn snap(c: Point2) -> (usize, usize) {
        Self::snap_coords(c.x, c.y)
    }
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        x.saturating_add(y.saturating_mul(self.width))
    }
    pub fn snap_coords(x: f32, y: f32) -> (usize, usize) {
        let x = (x / 32.) as usize;
        let y = (y / 32.) as usize;

        (x, y)
    }
    pub fn get(&self, x: usize, y: usize) -> Option<Material> {
        if x < self.width {
            self.mats.get(self.idx(x, y)).cloned()
        } else {
            None
        }
    }
    pub fn is_solid(&self, x: usize, y: usize) -> bool {
        self.get(x, y).map(|m| m.solid()).unwrap_or(true)
    }
    pub fn insert(&mut self, x: usize, y: usize, mat: Material) {
        if x < self.width {
            let i = self.idx(x, y);
            if let Some(m) = self.mats.get_mut(i) {
                *m = mat;
            }
        }
    }
    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        for (i, mat) in self.mats.iter().enumerate() {
            let x = (i % self.width) as f32 * 32.;
            let y = (i / self.width) as f32 * 32.;

            mat.draw(ctx, assets, x, y)?;
        }
        Ok(())
    }
}
