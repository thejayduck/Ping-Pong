use std::sync::Arc;
use std::time::Instant;
use std::{env, path};

use ball::Ball;
use ggez::audio::{SoundSource, Source};
use ggez::event::{self, EventHandler, KeyCode};
use ggez::graphics::{self, Color, DrawParam, Drawable, Mesh, Rect, TextFragment};
use ggez::{Context, ContextBuilder, GameResult};
use glam::{vec2, Vec2};
use player::Player;

mod ball;
mod player;

enum PlayerDirection {
    PlayerOne(Direction),
    PlayerTwo(Direction),
}

pub enum Direction {
    Up,
    Down,
}

enum GameState {
    Play,
    Over { player_one_won: bool, time: Instant },
}

impl PlayerDirection {
    pub fn from_keycode(key: KeyCode) -> Option<PlayerDirection> {
        match key {
            KeyCode::W => Some(PlayerDirection::PlayerOne(Direction::Up)),
            KeyCode::S => Some(PlayerDirection::PlayerOne(Direction::Down)),
            KeyCode::Up => Some(PlayerDirection::PlayerTwo(Direction::Up)),
            KeyCode::Down => Some(PlayerDirection::PlayerTwo(Direction::Down)),
            _ => None,
        }
    }
}

fn intersects(circle_pos: Vec2, circle_radius: f32, rect_pos: Vec2, rect_size: Vec2) -> bool {
    let mut circle_distance = ggez::mint::Point2 { x: 0.0, y: 0.0 };

    circle_distance.x = (circle_pos[0] - rect_pos[0]).abs();
    circle_distance.y = (circle_pos[1] - rect_pos[1]).abs();

    if circle_distance.x > (rect_size[0] / 2.0 + circle_radius) {
        return false;
    }
    if circle_distance.y > (rect_size[1] / 2.0 + circle_radius) {
        return false;
    }

    if circle_distance.x <= rect_size[0] / 2.0 {
        return true;
    }
    if circle_distance.y <= rect_size[1] / 2.0 {
        return true;
    }

    let corner_distance = (circle_distance.x - rect_size[0] / 2.0).powi(2)
        + (circle_distance.y - rect_size[1] / 2.0).powi(2);

    corner_distance <= circle_radius.powi(2)
}

fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (mut ctx, event_loop) = ContextBuilder::new("ping_pong", "TheJayDuck")
        .add_resource_path(resource_dir)
        .window_setup(ggez::conf::WindowSetup::default().title("Ping Pong by Jay"))
        .build()
        .expect("aieee, could not create ggez context!");

    let game = MyGame::new(&mut ctx);
    event::run(ctx, event_loop, game);
}

struct MyGame {
    player_one: Player,
    player_two: Player,

    ball: Ball,

    game_state: GameState,

    audio_hit: Source,
    audio_lose: Source,
    audio_wall: Source,
}

fn load_audio(ctx: &mut Context, path: &str, volume: f32) -> GameResult<Source> {
    let mut source = Source::new(ctx, path)?;
    source.set_volume(volume);
    Ok(source)
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        let (width, height) = graphics::drawable_size(ctx);

        let mesh = Mesh::new_rounded_rectangle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            Rect {
                x: 0.0,
                y: 0.0,
                w: 10.0,
                h: 200.0,
            },
            20.0,
            Color::WHITE,
        )
        .unwrap();

        let mesh_arc = Arc::new(mesh);
        let mesh_ball = Mesh::new_circle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            [5.0, 5.0],
            5.0,
            0.1,
            Color::WHITE,
        )
        .unwrap();

        MyGame {
            game_state: GameState::Play,
            player_one: Player::new(16.0, mesh_arc.clone()),
            player_two: Player::new(width - 16.0, mesh_arc),
            ball: Ball {
                mesh: mesh_ball,
                pos: vec2(width / 2.0, height / 2.0),
                vel: vec2(150.0, 0.0),
            },

            audio_hit: load_audio(ctx, "/sfx/hit.ogg", 0.5).unwrap(),
            audio_lose: load_audio(ctx, "/sfx/lose.ogg", 0.2).unwrap(),
            audio_wall: load_audio(ctx, "/sfx/wall.ogg", 0.5).unwrap(),
        }
    }

    pub fn play_update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let (width, height) = graphics::drawable_size(ctx);

        self.ball.pos += self.ball.vel / Vec2::splat(60.0);

        self.player_one.update_collision(
            ctx,
            self.ball.pos,
            &mut self.ball.vel,
            &mut self.audio_hit,
        );

        self.player_two.update_collision(
            ctx,
            self.ball.pos,
            &mut self.ball.vel,
            &mut self.audio_hit,
        );

        if self.ball.pos.y <= 0.0 || self.ball.pos.y >= height {
            self.ball.vel.y *= -1.0;
            self.audio_wall.play(ctx)?;
        }

        if self.ball.pos.x >= width {
            self.player_one.score += 1;
            self.audio_lose.play(ctx)?;

            self.game_state = GameState::Over {
                player_one_won: true,
                time: Instant::now(),
            }
        }

        if self.ball.pos.x <= 0.0 {
            self.player_two.score += 1;
            self.audio_lose.play(ctx)?;

            self.game_state = GameState::Over {
                player_one_won: false,
                time: Instant::now(),
            }
        }
        self.player_one.move_to_velocity(ctx);
        self.player_two.move_to_velocity(ctx);

        Ok(())
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let (width, height) = graphics::drawable_size(ctx);
        match self.game_state {
            GameState::Play => self.play_update(ctx),
            GameState::Over { time, .. } => {
                if time.elapsed().as_secs_f32() > 3.0 {
                    self.game_state = GameState::Play;
                    self.ball.pos = vec2(width / 2.0, height / 2.0);
                    self.ball.vel.x = 150.0;

                    self.player_one.target_vel = 0.0;
                    self.player_one.vel = 0.0;

                    self.player_two.target_vel = 0.0;
                    self.player_two.vel = 0.0;
                }
                Ok(())
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let (width, height) = graphics::drawable_size(ctx);

        graphics::clear(ctx, Color::BLACK);

        self.player_one.draw(ctx)?;
        self.player_two.draw(ctx)?;

        self.ball.draw(ctx)?;

        let score_text = graphics::Text::new(
            TextFragment::new(format!(
                "{0} | {1}",
                self.player_one.score, self.player_two.score
            ))
            .scale(20.0),
        );

        let score_text_dim = score_text.dimensions(ctx);

        score_text.draw(
            ctx,
            DrawParam::new()
                .dest([width / 2.0, 0.0])
                .offset(vec2(score_text_dim.w, 0.0) / Vec2::splat(2.0)),
        )?;

        if let GameState::Over { player_one_won, .. } = self.game_state {
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w: width,
                    h: height,
                },
                Color {
                    a: 0.5,
                    ..Color::RED
                },
            )?
            .draw(ctx, DrawParam::new())?;

            let win_text = graphics::Text::new(
                TextFragment::new(if player_one_won {
                    "Player 1 Won!"
                } else {
                    "Player 2 Won!"
                })
                .scale(20.0),
            );

            let win_text_dim = win_text.dimensions(ctx);

            win_text.draw(
                ctx,
                DrawParam::new()
                    .dest([width / 2.0, height / 2.0])
                    .offset(vec2(win_text_dim.w, win_text_dim.h) / Vec2::splat(2.0)),
            )?;
        }

        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymods: event::KeyMods,
        _repeat: bool,
    ) {
        if let GameState::Play = self.game_state {
            if let Some(direction) = PlayerDirection::from_keycode(keycode) {
                match direction {
                    PlayerDirection::PlayerOne(direction) => {
                        self.player_one.handle_input_down(direction)
                    }
                    PlayerDirection::PlayerTwo(direction) => {
                        self.player_two.handle_input_down(direction)
                    }
                }
            }
        }
    }
    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: event::KeyMods) {
        if let GameState::Play = self.game_state {
            if let Some(direction) = PlayerDirection::from_keycode(keycode) {
                match direction {
                    PlayerDirection::PlayerOne(_) => self.player_one.handle_input_up(),
                    PlayerDirection::PlayerTwo(_) => self.player_two.handle_input_up(),
                }
            }
        }
    }
}
