use ::*;

#[derive(Debug, Serialize, Deserialize)]
/// All the objects in the current world
pub struct World {
    pub(super) player: Object,
    pub(super) level: Level,
    pub(super) bullets: Vec<Object>,
    pub(super) holes: Vec<Object>,
}

include!("material_macro.rs");

mat!{
    MISSING = Missing
    Grass = 0, Grass, false,
    Wall = 1, Wall, true,
    Floor = 2, Floor, false,
    Missing = 404, Missing, true,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    grid: [[Material; 32]; 32],
}

impl Level {
    pub fn new() -> Self {
        Level {
            grid: [[Material::Grass; 32]; 32]
        }
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
        self.grid[x][y]
    }
    pub fn insert(&mut self, x: usize, y: usize, mat: Material) {
        self.grid[x][y] = mat;
    }
    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        for (i, vert) in self.grid.iter().enumerate() {
            for (j, mat) in vert.iter().enumerate() {
                let x = i as f32 * 32.;
                let y = j as f32 * 32.;

                mat.draw(ctx, assets, x, y)?;
            }
        }
        Ok(())
    }
}
