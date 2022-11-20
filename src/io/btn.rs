use crate::util::Point2;
use super::tex::{Assets, PosText};

use ggez::{GameResult, Context};
use ggez::graphics::{Drawable, Color, Rect, Mesh, DrawMode, DrawParam, Canvas};
use nalgebra::coordinates::XY;

pub struct Button<T> {
    width: f32,
    height: f32,
    pub callback: T,
    text: PosText,
    mesh: Mesh
}

impl<T> Button<T> {
    pub fn new(ctx: &mut Context, assets: &Assets, rect: Rect, text: &str, callback: T) -> GameResult<Self> {
        let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color{r: 0.5, g: 0.5, b: 0.75, a: 1.})?;
        let text = assets.text(point!(rect.x + rect.w / 2., rect.y + rect.h / 2.)).and_text(text).centered();

        Ok(Button{
            text,
            mesh,
            callback,
            width: rect.w,
            height: rect.h,
        })
    }
    pub fn draw(&self, canvas: &mut Canvas) {
        self.mesh.draw(canvas, DrawParam::new());
        self.text.draw_text(canvas);
    }
    pub fn in_bounds(&self, p: Point2) -> bool {
        let XY{x, y} = *self.text.pos;
        let (w, h) = (self.width / 2., self.height / 2.);

        p.x >= x - w  && p.x < x + w && p.y >= y - h && p.y < y + h
    }
}
