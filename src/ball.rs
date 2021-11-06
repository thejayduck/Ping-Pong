use ggez::{
    graphics::{DrawParam, Drawable, Mesh},
    Context, GameResult,
};
use glam::Vec2;

pub struct Ball {
    pub mesh: Mesh,

    pub pos: Vec2,
    pub vel: Vec2,
}

impl Ball {
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.mesh.draw(
            ctx,
            DrawParam::new().dest(self.pos).offset([10.0 / 2.0, 0.0]),
        )
    }
}
