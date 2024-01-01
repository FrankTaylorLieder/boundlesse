#[allow(unused)]
use ggez::{
    conf::WindowMode,
    event::{self, EventHandler},
    glam::*,
    graphics::{self, Color, DrawMode, DrawParam, Mesh, Quad, Rect},
    mint::Point2,
    Context, ContextBuilder, GameError, GameResult,
};
use std::time::SystemTime;

fn now() -> u128 {
    let duration_since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    duration_since_epoch.as_nanos() / 1000
}
const DEFAULT_WINDOW_SIZE: (f32, f32) = (2000.0, 1500.0);

const BG_COLOR: Color = Color::WHITE;
const CELL_COLOR: Color = Color::BLACK;

struct State {
    running: bool,
}

impl State {
    pub fn new(_ctx: &mut Context) -> Self {
        State { running: true }
    }
}

impl EventHandler<GameError> for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        println!("Draw requested...");

        let start = now();
        let mut live = 0;

        let mut canvas = graphics::Canvas::from_frame(ctx, BG_COLOR);

        let cs = 2.0;
        let lines = 100;
        for x in 0..500 {
            for y in 0..lines {
                live += 1;
                // let rect = Mesh::new_rectangle(
                //     ctx,
                //     DrawMode::fill(),
                //     Rect::new(x as f32 * cs, y as f32 * cs, cs, cs),
                //     CELL_COLOR,
                // )?;
                // canvas.draw(&rect, graphics::DrawParam::from(Point2 { x: 0.0, y: 0.0 }));

                canvas.draw(
                    &Quad,
                    DrawParam::default()
                        .color(CELL_COLOR)
                        .scale([cs, cs])
                        .dest([x as f32 * cs, y as f32 * cs]),
                );
            }
        }

        println!("Draw finished: {} objects in {}", live, now() - start);
        canvas.finish(ctx)?;

        println!("Draw done: {}", now() - start);

        Ok(())
    }
}

fn main() -> GameResult {
    println!("Starting rusty-life lock up test...");

    let (mut ctx, event_loop) = ContextBuilder::new("Rusty Life", "Frank Taylor")
        .window_mode(
            WindowMode::default()
                .dimensions(DEFAULT_WINDOW_SIZE.0, DEFAULT_WINDOW_SIZE.1)
                .resizable(true),
        )
        .build()?;

    let mut state = State::new(&mut ctx);

    state.running = true;

    event::run(ctx, event_loop, state);
}
