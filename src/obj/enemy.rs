use ggez::{Context, GameResult};
use ggez::graphics;

use obj::Object;
use ::{Assets, Sprite, RED};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    obj: Object,
}

impl Enemy {
    pub fn new(obj: Object) -> Enemy {
        Enemy {
            obj
        }
    }
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        graphics::set_color(ctx, RED)?;
        self.obj.draw(ctx, a.get_img(Sprite::Person))
    }
}
