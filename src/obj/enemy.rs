use crate::util::{BLUE, Vector2, Point2};

use ggez::{
    Context, GameResult,
    graphics::{Color, DrawMode, DrawParam, Canvas, Mesh},
};
use rand::{thread_rng, Rng};

use crate::{
    util::{angle_from_vec, angle_to_vec},
    io::{
        snd::MediaPlayer,
        tex::{Assets, },
    },
    DELTA,
    world::{Grid, Palette},
};

use super::{Object, player::Player};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Behaviour {
    #[serde(with = "crate::io::save::Point2DefVec")]
    pub path: Vec<Point2>,
    pub cyclical_path: bool,
    #[serde(skip)]
    cur_target: Option<Point2>,
}

impl Behaviour {
    pub fn chasing(&self) -> bool {
        self.cur_target.is_some()
    }
    pub fn chase_then_wander(&mut self, target: Point2) {
        self.path = Vec::new();
        self.cyclical_path = true;
        self.cur_target = Some(target);
    }
    pub fn patrol_path(&mut self, path: Vec<Point2>) {
        self.path = path;
        self.cyclical_path = true;
        self.cur_target = None;
    }
    pub fn path_then_wander(&mut self, path: Vec<Point2>) {
        self.cur_target = None;
        self.path = path;
        self.cyclical_path = false;
    }
    pub fn go_to_then_go_back(&mut self, cur_pos: Point2, target: Point2) {
        self.cur_target = Some(target);
        self.path.insert(0, cur_pos);
    }
}

impl Default for Behaviour {
    fn default() -> Self {
        Behaviour { path: vec![], cur_target: None, cyclical_path: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OldEnemy {
    pub pl: Player,
    #[serde(skip)]
    pub behaviour: Behaviour,
}

impl From<OldEnemy> for Enemy {
    fn from(OldEnemy {pl, behaviour}: OldEnemy) -> Self {
        Enemy {pl, behaviour }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub pl: Player,
    pub behaviour: Behaviour,
}

pub const VISIBILITY: f32 = ::std::f32::consts::FRAC_PI_4;

impl Enemy {
    pub fn new(obj: Object) -> Enemy {
        Enemy {
            pl: Player::new(obj),
            behaviour: Behaviour::default(),
        }
    }
    /// Draws two lines from the enemy indicating the field of vision
    pub fn draw_visibility_cone(&self, ctx: &mut Context, canvas: &mut Canvas, length: f32) -> GameResult<()> {
        let Object{pos, rot} = self.pl.obj;
        let dir1 = angle_to_vec(rot - VISIBILITY);
        let dir2 = angle_to_vec(rot + VISIBILITY);

        let mesh = Mesh::new_polyline(ctx, DrawMode::stroke(1.5), &[pos + (length * dir1), pos, pos + (length * dir2)], BLUE)?;

        canvas.draw(&mesh, DrawParam::default());
        Ok(())
    }
    #[inline]
    pub fn draw(&self, canvas: &mut Canvas, a: &Assets, color: Color) {
        self.pl.draw(canvas, a, "common/enemy", color);
    }
    /// Look in the direction of a given vector
    /// ### Returns
    /// `true` if the enemy is now facing that direction
    fn look_towards(&mut self, dist: Vector2) -> bool {
        let dir = angle_to_vec(self.pl.obj.rot);

        let rotation = dir.angle(&dist);

        const ROTATION: f32 = 6. * DELTA;

        if rotation > ROTATION {
            if dir.perp(&dist) > 0. {
                self.pl.obj.rot += ROTATION;
            } else {
                self.pl.obj.rot -= ROTATION;
            }
            false
        } else {
            self.pl.obj.rot = angle_from_vec(dist);
            true
        }
    }
    pub fn update<F: FnOnce() -> Point2>(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer, wander_finder: F) -> GameResult<()> {
        if let Some(wep) = self.pl.wep.get_active_mut() {
            wep.update(ctx, mplayer)?;
            if wep.cur_clip == 0 && wep.loading_time == 0. {
                wep.reload(ctx, mplayer)?;
            }
        }
        match &mut self.behaviour.cur_target {
            t @ None if !self.behaviour.path.is_empty() => {
                let next_node = self.behaviour.path.remove(0);

                if self.behaviour.cyclical_path {
                    self.behaviour.path.push(next_node);
                } else if self.behaviour.path.is_empty() {
                    // If the path is now empty, we set the empty path as cyclical so as to wander around
                    self.behaviour.cyclical_path = true;
                }

                *t = Some(next_node);
            }
            // Wander around if the path is empty but also flagged as cyclical
            t @ None if self.behaviour.cyclical_path && thread_rng().gen_range(0..10) == 0 => *t = Some(wander_finder()),
            // Stare intensely if there's no path and we're not wandering
            None => (),
            Some(_) => (),
        }
        if let Some(t) = self.behaviour.cur_target {
            let dist = t - self.pl.obj.pos;

            if self.look_towards(dist) {
                let distance = dist.norm();
                const CHASE_SPEED: f32 = 100. * DELTA;
        
                if distance >= CHASE_SPEED {
                    let displace = CHASE_SPEED * dist / distance;
                    self.pl.obj.pos += displace;
                } else {
                    // We have reached the target
                    self.behaviour.cur_target = None;
                }
            }
        }
        Ok(())
    }
    pub fn can_see(&self, p: Point2, palette: &Palette, grid: &Grid) -> bool {
        let dist = p-self.pl.obj.pos;
        let dir = angle_to_vec(self.pl.obj.rot);

        dir.angle(&dist) <= VISIBILITY && grid.ray_cast(palette, self.pl.obj.pos, dist, true).full()
    }
}
