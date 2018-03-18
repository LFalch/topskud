use std::f32::consts::PI;

use super::*;
use ::{InputState, DELTA, angle_to_vec, BLUE, RED, GREEN};

const DRAG: f32 = 0.0029;

fn zero() -> Vector2 {
    Vector2::new(0., 0.)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Car {
    pub obj: Object,
    #[serde(serialize_with = "::save::vec_ser", deserialize_with = "::save::vec_des")]
    velocity: Vector2,
    engine: f32,
    brakes: f32,
    #[serde(skip, default = "zero")]
    acc: Vector2,
    #[serde(skip, default = "zero")]
    drag: Vector2,
}

impl Car {
    pub fn new(x: f32, y: f32, engine: f32, brakes: f32) -> Self {
        Car {
            obj: Object::new(Point2::new(x, y)),
            velocity: Vector2::new(0., 0.),
            engine,
            brakes,
            acc: zero(),
            drag: zero(),
        }
    }
    pub fn update(&mut self, input: &InputState) {
        let ang = angle_to_vec(self.obj.rot);
        let speed_forwards = self.velocity.dot(&ang);

        self.obj.rot += input.hor() * 2. * PI * 0.001 * speed_forwards * DELTA;

        self.drag = -DRAG * self.velocity.norm() * self.velocity;
        self.acc;
        match input.ver {
            -1 => self.acc = ang * self.engine,
            1 if self.velocity.norm() > 0. => self.acc = -self.brakes * self.velocity.normalize(),
            _ => self.acc = zero(),
        }

        self.obj.pos += self.velocity * DELTA + 0.5 * (self.acc+self.drag) * DELTA * DELTA;
        self.velocity += (self.acc+self.drag) * DELTA;
    }
    pub fn draw_lines(&self, ctx: &mut Context) -> GameResult<()> {
        let vel = self.obj.pos+self.velocity;
        let acc = self.obj.pos+self.acc;
        let drag = self.obj.pos+self.drag;

        graphics::set_color(ctx, BLUE)?;
        graphics::line(ctx, &[self.obj.pos, vel], 2.)?;

        graphics::set_color(ctx, GREEN)?;
        graphics::line(ctx, &[self.obj.pos, acc], 2.)?;

        graphics::set_color(ctx, RED)?;
        graphics::line(ctx, &[self.obj.pos, drag], 2.)
    }
}
