use ggez::{Context, GameResult, graphics::{self, WHITE, DrawMode, MeshBuilder, DrawParam, FillOptions}};
use std::f32::consts::PI;

use crate::{
    util::{angle_to_vec, Vector2},
    game::{
        DELTA,
        world::{Grid, Palette},
    },
    io::{
        snd::MediaPlayer,
        tex::{Assets, },
    },
};
use super::{Object, player::Player, enemy::Enemy, health::Health};

#[derive(Debug, Default, Clone, Copy)]
pub struct Utilities {
    pub grenades: u8,
}

#[derive(Debug, Clone)]
pub struct Grenade {
    pub obj: Object,
    pub vel: Vector2,
    pub fuse: f32,
}

const DEC: f32 = 1.4;

impl Grenade {
    #[inline]
    pub fn apply_damage(&self, health: &mut Health, high: bool) {
        health.weapon_damage(if high { 105.} else {55.}, 0.85);
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets, palette: &Palette, grid: &Grid) -> GameResult<()> {
        if self.fuse > 2.*DELTA {
            let img = a.get_img(ctx, "weapons/pineapple");
            self.obj.draw(ctx, &*img, WHITE)
        } else {
            let expl_img = a.get_img(ctx, "weapons/explosion");
            // expl_img.set_wrap(WrapMode::Mirror,WrapMode::Mirror);
            let mut explosion = Vec::new();
            for i in 0..180 {
                let cast = grid.ray_cast(palette, self.obj.pos, angle_to_vec(((i*2) as f32)*PI/180.)*144., true);
                explosion.push(cast.into_point());
            }

            let explosion_mesh = MeshBuilder::new()
                .polygon(DrawMode::Fill(FillOptions::even_odd()), &explosion, WHITE)?
                // .texture(expl_img.clone())n
                .build(ctx)?;
            graphics::draw(ctx, &explosion_mesh, DrawParam::default())
            // let mut explosion_mesh = Mesh::new_polygon(ctx, DrawMode::Fill(FillOptions::even_odd()), &explosion, Color::from_rgba(235, 91, 20, 255))?;
            // self.obj.draw(ctx, &*expl_img, WHITE)
            // self.obj.draw(ctx, &explosion_mesh, DrawParam::default())
        }
    }
    pub fn update(&mut self, palette: &Palette, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> Option<Explosion> {
        let start = self.obj.pos;
        let d_vel = -DEC * self.vel * DELTA;
        let d_pos = 0.5 * DELTA * d_vel + self.vel * DELTA;
        self.vel += d_vel;
        if self.fuse > DELTA {
            self.fuse -= DELTA;
        } else {
            self.fuse = 0.;

            let player_hit;
            let mut enemy_hits = Vec::new();

            let d_player = player.obj.pos-start;
            if d_player.norm() < 144. && grid.ray_cast(palette, start, d_player, true).full() {
                self.apply_damage(&mut player.health, d_player.norm() <= 64.);
                player_hit = true;
            } else {
                player_hit = false;
            }

            for (i, enem) in enemies.iter_mut().enumerate().rev() {
                let d_enemy = enem.pl.obj.pos - start;
                if d_enemy.norm() < 144. && grid.ray_cast(palette, start, d_enemy, true).full() {
                    self.apply_damage(&mut enem.pl.health, d_enemy.norm() <= 64.);
                    enemy_hits.push(i);
                }
            }

            return Some(Explosion{player_hit, enemy_hits});
        }

        let closest_p = Grid::closest_point_of_line_to_circle(start, d_pos, player.obj.pos);
        let r_player = player.obj.pos - closest_p;
        if r_player.norm() <= 16. {
            self.vel -= 2.*self.vel.dot(&r_player)/r_player.norm_squared() * r_player;
            let clip = (start + d_pos) - closest_p;

            self.obj.pos = closest_p + clip -  2. * clip.dot(&r_player)/r_player.norm_squared()*r_player;
            return None;
        }
        for enem in enemies.iter_mut() {
            let closest_e = Grid::closest_point_of_line_to_circle(start, d_pos, enem.pl.obj.pos);
            let r_enemy = enem.pl.obj.pos - closest_e;
            if r_enemy.norm() <= 16. {
                self.vel -= 2.*self.vel.dot(&r_enemy)/r_enemy.norm_squared() * r_enemy;
                let clip = (start + d_pos) - closest_p;

                self.obj.pos = closest_e + clip - 2. * clip.dot(&r_enemy)/r_enemy.norm_squared()*r_enemy;
                return None;
            }
        }
        let cast = grid.ray_cast(palette, start, d_pos, true);
        self.obj.pos = cast.into_point();
        if let Some(to_wall) = cast.half_vec() {
            let clip = cast.clip();
            self.obj.pos += clip -  2. * clip.dot(&to_wall)/to_wall.norm_squared() * to_wall;
            self.vel -= 2. * self.vel.dot(&to_wall)/to_wall.norm_squared() * to_wall;
        }
        None
    }
}

impl Utilities {
    pub fn throw_grenade(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<Option<GrenadeMaker>> {
        if self.grenades > 0 {
            self.grenades -= 1;

            mplayer.play(ctx, "throw")?;
            Ok(Some(GrenadeMaker(620.)))
        } else {
            mplayer.play(ctx, "cock")?;
            Ok(None)
        }
    }
}

pub struct GrenadeMaker(f32);
impl GrenadeMaker {
    pub fn make(self, mut obj: Object) -> Grenade {
        let vel = angle_to_vec(obj.rot) * self.0;
        obj.rot = 0.;
        Grenade {
            fuse: 1.5,
            vel,
            obj,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Explosion{
    pub player_hit: bool,
    pub enemy_hits: Vec<usize>,
}
