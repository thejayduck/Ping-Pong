use std::sync::Arc;

use ggez::{
    audio::{SoundSource, Source},
    graphics::{self, DrawParam, Drawable, Mesh},
    timer, Context, GameResult,
};
use glam::{vec2, Vec2};

use crate::{intersects, Direction};

pub struct Player {
    pos: Vec2,
    vel: f32,
    target_vel: f32,
    mesh: Arc<Mesh>,

    pub score: u32,
}

impl Player {
    pub fn new(x: f32, mesh: Arc<Mesh>) -> Self {
        Player {
            pos: vec2(x, 0.0),
            vel: 0.0,
            target_vel: 0.0,
            mesh,
            score: 0,
        }
    }

    pub fn update_collision(
        &self,
        ctx: &mut Context,
        ball_pos: Vec2,
        ball_vel: &mut Vec2,
        audio: &mut Source,
    ) {
        if self.intersects_ball(ball_pos) {
            let y_sign = ball_vel.y.signum();

            let diff = ball_pos.y - (self.pos.y + 200.0 / 2.0);
            ball_vel.x *= -1.05;
            ball_vel.y = diff * -y_sign;

            audio.play(ctx).unwrap();
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.mesh.draw(
            ctx,
            DrawParam::new().dest(self.pos).offset([10.0 / 2.0, 0.0]),
        )
    }

    pub fn handle_input_down(&mut self, direction: Direction) {
        const SPEED: f32 = 30.0;

        self.target_vel = match direction {
            Direction::Up => -SPEED,
            Direction::Down => SPEED,
        };
    }

    pub fn move_to_velocity(&mut self, ctx: &mut Context) {
        let (_, height) = graphics::drawable_size(ctx);
        const SMOOTH: f32 = 0.2;

        self.vel += (self.target_vel - self.vel) * SMOOTH * timer::delta(ctx).as_secs_f32();
        self.pos.y += self.vel;

        if self.pos.y < 0.0 {
            self.pos.y = 0.0;
            self.target_vel = 0.0;
            self.vel = 0.0;
        }
        if self.pos.y > height - 200.0 {
            self.pos.y = height - 200.0;
            self.target_vel = 0.0;
            self.vel = 0.0;
        }
    }

    pub fn handle_input_up(&mut self) {
        self.target_vel = 0.0;
    }

    fn intersects_ball(&self, ball_pos: Vec2) -> bool {
        intersects(
            ball_pos + Vec2::splat(5.0),
            5.0,
            vec2(self.pos.x + 10.0 / 2.0, self.pos.y + 200.0 / 2.0),
            vec2(10.0, 200.0),
        )
    }
}
