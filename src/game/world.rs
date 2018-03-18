use ::*;

#[derive(Debug, Serialize, Deserialize)]
/// All the objects in the current world
pub struct World {
    pub(super) player: Object,
    pub(super) level: Level,
    pub(super) bullets: Vec<Object>,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum Material {
    Grass = 0,
    Wall = 1,
    Floor = 2,
}

use serde::{Serialize, Deserialize, Serializer, Deserializer};

impl Serialize for Material {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        serializer.serialize_u16(*self as u16)
    }
}

impl<'de> Deserialize<'de> for Material {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> {
        <u16>::deserialize(deserializer).map(
            |n| match n {
                0 => Material::Grass,
                1 => Material::Wall,
                2 => Material::Floor,
                _ => unimplemented!(),
            }
        )
    }
}

impl Material {
    pub fn get_spr(&self) -> Sprite {
        match *self {
            Material::Grass => Sprite::Grass,
            Material::Wall => Sprite::Wall,
            Material::Floor => Sprite::Floor,
        }
    }
    pub fn solid(&self) -> bool {
        match *self {
            Material::Grass => false,
            Material::Wall => true,
            Material::Floor => false,
        }
    }
    pub fn draw(&self, ctx: &mut Context, assets: &Assets, x: f32, y: f32) -> GameResult<()> {
        let img = assets.get_img(self.get_spr());
        let drawparams = graphics::DrawParam {
            dest: Point2::new(x, y),
            .. Default::default()
        };
        graphics::draw_ex(ctx, img, drawparams)
    }
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
