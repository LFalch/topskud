use crate::{
    io::tex::Assets,
    util::{Point2, Vector2},
};
use ggez::{graphics::{self, Image}, Context, GameResult};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;
use std::fs::File;
use std::io::Read;
use std::cell::Ref;

#[derive(Debug)]
pub struct Mat {
    spr: Box<str>,
    props: MaterialProperties
}

lazy_static! {
    static ref MATS: RwLock<HashMap<String, Mat>> = {
        RwLock::new(HashMap::with_capacity(10))
    };
}

fn ensure(mat: &str) {
    if !MATS.read().unwrap().contains_key(mat) {
        let props = if let Ok(mut f) = File::open(format!("resources/materials/{}.mat", mat)) {
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();

            toml::from_str(&s).unwrap()
        } else {
            MaterialProperties::default()
        };
        let mat_data = Mat { spr: format!("materials/{}", mat).into_boxed_str(), props};

        MATS.write().unwrap().insert(mat.to_owned(), dbg!(mat_data));
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct MaterialProperties {
    solid: bool,
}

#[inline]
pub fn is_solid(mat: &str) -> bool {
    ensure(mat);

    MATS.read().unwrap()[mat].props.solid
}

#[inline]
pub fn get_img<'a>(ctx: &mut Context, assets: &'a Assets, mat: &str) -> Ref<'a, Image> {
    ensure(mat);

    assets.get_img(ctx, &MATS.read().unwrap()[mat].spr)
}

#[derive(Debug, Clone)]
pub struct Palette {
    materials: Box<[&'static str]>,
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            materials: Box::new([
                "grass",
                "wall",
                "floor",
                "dirt",
                "asphalt",
                "sand",
                "concrete",
                "wood_floor",
                "stairs",
            ])
        }
    }
}

impl Palette {
    pub fn new(mut mats: Vec<&'static str>) -> Self {
        // Fix dedup, use some sets
        mats.dedup();

        Palette {
            materials: mats.into_boxed_slice()
        }
    }
    pub fn and(self, other: &Self) -> Self {
        let Palette{materials} = self;
        let mut mats = materials.to_vec();

        mats.extend(other.materials.iter().copied());
        // Fix dedup, use some sets
        mats.dedup();

        Palette {
            materials: mats.into_boxed_slice(),
        }
    }
    pub fn draw_mat(&self, i: u8, ctx: &mut Context, assets: &Assets, x: f32, y: f32, dp: graphics::DrawParam) -> GameResult<()> {
        let mat = self.materials[i as usize];

        let img = get_img(ctx, assets, mat);
        graphics::draw(ctx, &*img, (Point2::from(dp.dest) + Vector2::new(x, y),))
    }
    pub fn is_solid(&self, i: u8) -> bool {
        is_solid(self.materials[i as usize])
    }
    #[inline]
    pub fn get(&self, i: u8) -> Option<&str> {
        self.materials.get(i as usize).copied()
    }
    #[inline]
    pub fn find(&self, mat: &str) -> Option<u8> {
        self.materials.iter().position(|s| &mat == s).map(|i| i as u8)
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.materials.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
}
