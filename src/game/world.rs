use crate::{
    util::{Point2, Vector2},
    io::tex::{Assets, },
    io::save::Point2Def,
    obj::{
        Object,
        player::Player,
        enemy::Enemy,
        health::Health,
        bullet::Bullet,
        grenade::Grenade,
        weapon::{WeaponInstance, WeaponDrop, WEAPONS},
        pickup::Pickup,
        decoration::DecorationObj,
    }
};
use ggez::{
    Context, GameResult,
    graphics,
    error::GameError,
};

use std::path::Path;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};

use ::bincode;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug)]
/// All the objects in the current world
pub struct World {
    pub player: Player,
    pub grid: Grid,
    pub exit: Option<Point2>,
    pub intels: Vec<Point2>,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet<'static>>,
    pub grenades: Vec<Grenade>,
    pub weapons: Vec<WeaponDrop<'static>>,
    pub decorations: Vec<DecorationObj>,
    pub pickups: Vec<Pickup>,
}

impl World {
    pub fn enemy_pickup(&mut self) {
        for enemy in &mut self.enemies {
            let mut dead = None;
            for (w, weapon) in self.weapons.iter().enumerate() {
                if (weapon.pos - enemy.pl.obj.pos).norm() <= 16. {
                    dead = Some(w);
                    break;
                }
            }
            if let Some(i) = dead {
                enemy.pl.wep = Some(WeaponInstance::from_drop(self.weapons.remove(i)));
            }
            let mut deads = Vec::new();
            for (p, pickup) in self.pickups.iter().enumerate() {
                if (pickup.pos - enemy.pl.obj.pos).norm() <= 16. {
                    deads.push(p);
                    break;
                }
            }
            for i in deads.into_iter() {
                let pickup = self.pickups.remove(i);
                pickup.apply(&mut enemy.pl.health);
            }
        }
    }
    pub fn player_pickup(&mut self) {
        let player = &mut self.player;
        if player.wep.is_none() {
            let mut dead = None;
            for (w, weapon) in self.weapons.iter().enumerate() {
                if (weapon.pos - player.obj.pos).norm() <= 16. {
                    dead = Some(w);
                    break;
                }
            }
            if let Some(i) = dead {
                player.wep = Some(WeaponInstance::from_drop(self.weapons.remove(i)));
            }
        }

        let mut deads = Vec::new();
        for (p, pickup) in self.pickups.iter().enumerate() {
            if (pickup.pos - player.obj.pos).norm() <= 16. {
                deads.push(p);
                break;
            }
        }
        for i in deads.into_iter() {
            let pickup = self.pickups.remove(i);
            pickup.apply(&mut player.health);
        }
    }
}

pub struct Statistics {
    pub hits: usize,
    pub misses: usize,
    pub enemies_left: usize,
    pub health_left: Health,
    pub level: Level,
    pub weapon: Option<WeaponInstance<'static>>,
}

include!("material_macro.rs");

mat!{
    MISSING = Missing
    Grass = 0, "materials/grass", false,
    Wall = 1, "materials/wall", true,
    Floor = 2, "materials/floor", false,
    Dirt = 3, "materials/dirt", false,
    Asphalt = 4, "materials/asphalt", false,
    Sand = 5, "materials/sand", false,
    Concrete = 6, "materials/concrete", true,
    WoodFloor = 7, "materials/wood_floor", false,
    Stairs = 8, "materials/stairs", false,
    Missing = 255, "materials/missing", true,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Entity {
    SimpleEnemy {
        obj: Object,
        weapon: usize,
    },
    Enemy {
        obj: Object,
        health: Health,
        weapon: usize,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLevel {
    pub grid: Grid,
    #[serde(with = "Point2Def")]
    pub start_point: Point2,
    #[serde(with = "opt_point")]
    pub exit: Option<Point2>,
    pub entities: Vec<Entity>,
}

mod opt_point {
    use serde::{Serialize, Deserialize, Serializer, Deserializer};
    use crate::util::Point2;

    #[inline]
    pub fn serialize<S: Serializer>(p: &Option<Point2>, s: S) -> Result<S::Ok, S::Error> {
        p.map(|p| (p.x, p.y)).serialize(s)
    }
    #[inline]
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Point2>, D::Error> {
        <Option<(f32, f32)>>::deserialize(d).map(|p| p.map(|(x, y)| Point2::new(x, y)))
    }
}


#[derive(Debug, Clone)]
pub struct Level {
    pub grid: Grid,
    pub start_point: Option<Point2>,
    pub enemies: Vec<Enemy>,
    pub exit: Option<Point2>,
    pub intels: Vec<Point2>,
    pub pickups: Vec<(Point2, u8)>,
    pub decorations: Vec<DecorationObj>,
    pub weapons: Vec<WeaponDrop<'static>>,
}

impl Level {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            grid: Grid::new(width, height),
            start_point: None,
            enemies: Vec::new(),
            exit: None,
            intels: Vec::new(),
            pickups: Vec::new(),
            decorations: Vec::new(),
            weapons: Vec::new(),
        }
    }
    pub fn load<P: AsRef<Path>>(path: P) -> GameResult<Self> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut ret = Level::new(0, 0);

        loop {
            let mut buf = String::with_capacity(16);
            reader.read_line(&mut buf)?;
            match &*buf.trim_end() {
                "" => continue,
                "GRD" => ret.grid = bincode::deserialize_from(&mut reader)
                .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "GRID" => {
                    let (w, grid): (usize, Vec<u16>) = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
                    ret.grid = Grid {
                        mats: grid.into_iter().map(|n| Material::from(n as u8)).collect(),
                        width: w as u16
                    }
                }
                "START" => ret.start_point = Some(
                    bincode::deserialize_from(&mut reader)
                        .map(|(x, y)| Point2::new(x, y))
                        .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?
                ),
                "ENEMIES" => ret.enemies = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "POINT GOAL" => ret.exit = Some(bincode::deserialize_from(&mut reader)
                    .map(|(x, y)| Point2::new(x, y))
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?),
                "INTELS" => ret.intels = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<(f32, f32)>| l.into_iter().map(|(x, y)| Point2::new(x, y)).collect())
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "DECORATIONS" => ret.decorations = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "PICKUPS" => ret.pickups = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<((f32, f32), u8)>| l.into_iter().map(|((x, y), i)| (Point2::new(x, y), i)).collect())
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "WEAPONS" => ret.weapons = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<((f32, f32), u8)>| l.into_iter().map(|((x, y), i)| WEAPONS[i as usize].make_drop(Point2::new(x, y))).collect())
                    .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?,
                "END" => break, 
                _ => return Err(GameError::ResourceLoadError("Bad section".to_string()))
            }
        }

        Ok(ret)
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) -> GameResult<()> {
        let mut file = File::create(path)?;

        writeln!(file, "GRD")?;
        bincode::serialize_into(&mut file, &self.grid)
            .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        if let Some(start) = self.start_point {
            writeln!(file, "\nSTART")?;
            bincode::serialize_into(&mut file, &(start.x, start.y))
            .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if !self.enemies.is_empty() {
            writeln!(file, "\nENEMIES")?;
            bincode::serialize_into(&mut file, &self.enemies)
            .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if let Some(p) = self.exit {
            writeln!(file, "\nPOINT GOAL")?;
            bincode::serialize_into(&mut file, &(p.x, p.y))
            .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if !self.intels.is_empty() {
            writeln!(file, "\nINTELS")?;
            let intels: Vec<_> = self.intels.iter().map(|p| (p.x, p.y)).collect();
            bincode::serialize_into(&mut file, &intels)
                .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if !self.decorations.is_empty() {
            writeln!(file, "\nDECORATIONS")?;
            bincode::serialize_into(&mut file, &self.decorations)
            .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if !self.pickups.is_empty() {
            writeln!(file, "\nPICKUPS")?;
            let pickups: Vec<_> = self.pickups.iter().map(|&(p, i)| ((p.x, p.y), i)).collect();
            bincode::serialize_into(&mut file, &pickups)
                .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }
        if !self.weapons.is_empty() {
            writeln!(file, "\nWEAPONS")?;
            let pickups: Vec<((f32, f32), u8)> = self.weapons.iter().map(|w| ((w.pos.x, w.pos.y), {
                let mut index = 0;
                for (i, wep) in WEAPONS.iter().enumerate() {
                    if wep.name == w.weapon.name {
                        index = i;
                        break
                    }  
                }
                index as u8
            })).collect();
            bincode::serialize_into(&mut file, &pickups)
                .map_err(|e| GameError::ResourceLoadError(format!("{:?}", e)))?;
        }

        writeln!(file, "\nEND")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid{
    width: u16,
    mats: Vec<Material>,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        Grid {
            width,
            mats: vec![Material::Grass; (width*height) as usize],
        }
    }
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }
    pub fn height(&self) -> u16 {
        self.mats.len() as u16 / self.width
    }
    pub fn widen(&mut self) {
        let width = self.width as usize;
        let height = self.height() as usize;
        self.mats.reserve_exact(height);
        for i in (1..=height).rev().map(|i| i * width) {
            self.mats.insert(i, Material::Grass);
        }
        self.width += 1;
    }
    pub fn thin(&mut self) {
        if self.width <= 1 {
            return
        }
        let width = self.width;
        for i in (1..=self.height()).rev().map(|i| i * width - 1) {
            self.mats.remove(i as usize);
        }
        self.width -= 1;
    }
    pub fn heighten(&mut self) {
        let new_len = self.mats.len() + self.width as usize;
        self.mats.reserve_exact(self.width as usize);
        self.mats.resize(new_len, Material::Grass);
    }
    pub fn shorten(&mut self) {
        let new_len = self.mats.len() - self.width as usize;
        if new_len == 0 {
            return
        }
        self.mats.truncate(new_len);
    }
    #[inline]
    pub fn snap(c: Point2) -> (u16, u16) {
        Self::snap_coords(c.x, c.y)
    }
    #[inline]
    fn idx(&self, x: u16, y: u16) -> usize {
        x.saturating_add(y.saturating_mul(self.width)) as usize
    }
    pub fn snap_coords(x: f32, y: f32) -> (u16, u16) {
        fn db32omin(n: f32) -> u16 {
            if n < 0. {
                std::u16::MAX
            } else {
                (n / 32.) as u16
            }
        }

        (db32omin(x), db32omin(y))
    }
    pub fn get(&self, x: u16, y: u16) -> Option<Material> {
        if x < self.width {
            self.mats.get(self.idx(x, y)).cloned()
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn is_solid_tuple(&self, (x, y): (u16, u16)) -> bool {
        self.is_solid(x, y)
    }
    pub fn is_solid(&self, x: u16, y: u16) -> bool {
        self.get(x, y).map(|m| m.solid()).unwrap_or(true)
    }
    pub fn insert(&mut self, x: u16, y: u16, mat: Material) {
        if x < self.width {
            let i = self.idx(x, y);
            if let Some(m) = self.mats.get_mut(i) {
                *m = mat;
            }
        }
    }
    pub fn ray_cast(&self, from: Point2, dist: Vector2, finite: bool) -> RayCast {
        let dest = from + dist;

        let mut cur = from;
        let mut to_wall = Vector2::new(0., 0.);
        let (mut gx, mut gy) = Self::snap(cur);
        let x_dir = Direction::new(dist.x);
        let y_dir = Direction::new(dist.y);

        loop {
            if finite && (cur - dest).dot(&dist) / dist.norm() >= 0. {
                break RayCast::n_full(dest);
            }

            let mat = self.get(gx, gy);

            if let Some(mat) = mat {
                if mat.solid() {
                    break RayCast::n_half(cur, dest-cur, to_wall);
                }
                if cur.x < 0. || cur.y < 0. {
                    break RayCast::n_off_edge(cur, dest-cur); 
                }
            } else {
                break RayCast::n_off_edge(cur, dest-cur);
            }

            let nearest_corner = Point2::new(x_dir.on(f32::from(gx) * 32.), y_dir.on(f32::from(gy) * 32.));
            let distance = nearest_corner - cur;

            let time = (distance.x/dist.x, distance.y/dist.y);

            if time.0 < time.1 {
                to_wall.x = dist.x.signum();
                to_wall.y = 0.;
                // Going along x
                cur.x = nearest_corner.x;
                cur.y += time.0 * dist.y;

                gx = if let Some(n) = x_dir.on_u16(gx) {
                    n
                } else {
                    break RayCast::n_off_edge(cur, dest-cur);
                }
            } else {
                if time.0 - time.1 < std::f32::EPSILON {
                    to_wall.x = dist.x.signum();
                    to_wall.y = dist.y.signum();
                } else {
                    to_wall.x = 0.;
                    to_wall.y = dist.y.signum();
                }
                // Going along y
                cur.y = nearest_corner.y;
                cur.x += time.1 * dist.x;

                gy = if let Some(n) = y_dir.on_u16(gy) {
                    n
                } else {
                    break RayCast::n_off_edge(cur, dest-cur);
                }
            }
        }
    }
    /// Closest point on a line segment to a circle
    pub fn closest_point_of_line_to_circle(line_start: Point2, line_dist: Vector2, circle_center: Point2) -> Point2 {
        let c = circle_center - line_start;

        let d_len = line_dist.norm();

        let c_on_d_len = c.dot(&line_dist) / d_len;

        if c_on_d_len < 0. {
            // Closest point is start point
            line_start
        } else if c_on_d_len <= d_len {
            // Closest point is betweeen start and end point
            let c_on_d = c_on_d_len / d_len * line_dist;
            line_start + c_on_d
        } else {
            // Closest point is end point
            line_start + line_dist
        }
    }
    /// Distance between a line section and a circle
    /// 
    /// The general formula for distance between a line and cirlcle here would be inadequate
    /// since here the line has a finite length so we need to check if the smalleset distance is in that finite line section.
    #[inline]
    pub fn distance_line_circle(line_start: Point2, line_dist: Vector2, circle_center: Point2) -> Vector2 {
        let closest_point = Self::closest_point_of_line_to_circle(line_start, line_dist, circle_center);

        circle_center - closest_point
    }
    /// Length of `distance_line_circle`
    #[inline]
    pub fn dist_line_circle(line_start: Point2, line_dist: Vector2, circle_center: Point2) -> f32 {
        Self::distance_line_circle(line_start, line_dist, circle_center).norm()
    }
    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        for (i, mat) in self.mats.iter().enumerate() {
            let x = f32::from(i as u16 % self.width) * 32.;
            let y = f32::from(i as u16 / self.width) * 32.;

            mat.draw(ctx, assets, x, y, Default::default())?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Pos,
    Neg,
}

impl Direction {
    #[inline]
    fn new(n: f32) -> Self {
        if n.is_sign_negative() {
            Direction::Neg
        } else {
            Direction::Pos
        }
    }
    #[inline]
    fn on_u16(self, n: u16) -> Option<u16> {
        match self {
            Direction::Pos => Some(n + 1),
            Direction::Neg => n.checked_sub(1),
        }
    }
    #[inline]
    fn on(self, n: f32) -> f32 {
        match self {
            Direction::Pos => n + 32.,
            Direction::Neg => n,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RayCast {
    result: RayCastResult,
    point: Point2,
    clip: Vector2,
}

#[derive(Debug, Copy, Clone)]
enum RayCastResult {
    Full,
    Half(Vector2),
    OffEdge,
}

impl RayCast {
    fn n_full(point: Point2) -> Self {
        RayCast{
            result: RayCastResult::Full,
            point,
            clip: Vector2::new(0., 0.)
        }
    }
    fn n_half(point: Point2, clip: Vector2, to_wall: Vector2) -> Self {
        RayCast{
            result: RayCastResult::Half(to_wall),
            point,
            clip,
        }
    }
    fn n_off_edge(point: Point2, clip: Vector2) -> Self {
        RayCast{
            result: RayCastResult::OffEdge,
            point,
            clip,
        }
    }

    pub fn full(self) -> bool {
        match self.result {
            RayCastResult::Full => true,
            _ => false,
        }
    }
    pub fn half(self) -> bool {
        match self.result {
            RayCastResult::Half(_) => true,
            _ => false,
        }
    }
    pub fn half_vec(self) -> Option<Vector2> {
        match self.result {
            RayCastResult::Half(v) => Some(v),
            _ => None,
        }
    }
    pub fn into_point(self) -> Point2 {
        let Self{point, ..} = self;
        point
    }
    pub fn clip(self) -> Vector2 {
        let Self{clip, ..} = self;
        clip
    }
}