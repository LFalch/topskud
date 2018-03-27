use super::tex::{Assets, PosText};

use ggez::{GameResult, Context};
use ggez::graphics::{Drawable, Color, Rect, Mesh, Point2, DrawMode, DrawParam};

pub struct Button {
    width: f32,
    height: f32,
    text: PosText,
    mesh: Mesh
}

impl Button {
    pub fn new(ctx: &mut Context, assets: &Assets, rect: Rect, text: &str) -> GameResult<Self> {
        let x2 = rect.x + rect.w;
        let y2 = rect.y + rect.h;
        let pts = [
            Point2::new(rect.x, rect.y),
            Point2::new(x2, rect.y),
            Point2::new(x2, y2),
            Point2::new(rect.x, y2),
        ];
        let mesh = Mesh::new_polygon(ctx, DrawMode::Fill, &pts)?;
        let text = assets.text(ctx, Point2::new(rect.x + rect.w / 2., rect.y + rect.h / 2.), text)?;
        Ok(Button{
            text,
            mesh,
            width: rect.w,
            height: rect.h,
        })
    }
    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let param = DrawParam {
            color: Some(Color{r: 0.5, g: 0.5, b: 0.75, a: 1.}),
            .. DrawParam::default()
        };
        self.mesh.draw_ex(ctx, param)?;
        self.text.draw_center(ctx)
    }
    pub fn in_bounds(&self, p: Point2) -> bool {
        let x1 = self.text.pos.x - self.width / 2.;
        let x2 = self.text.pos.x + self.width / 2.;
        let y1 = self.text.pos.y - self.height / 2.;
        let y2 = self.text.pos.y + self.height / 2.;
        p.x >= x1 && p.x < x2 && p.y >= y1 && p.y < y2
    }
}
