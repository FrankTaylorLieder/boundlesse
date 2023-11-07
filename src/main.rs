use ggez::glam::*;
use ggez::input::keyboard::KeyCode;
use ggez::input::keyboard::KeyInput;
use ggez::{
    conf::WindowMode,
    event::{self, EventHandler, MouseButton},
    graphics::{self, Color, DrawMode, Mesh, Rect, Text},
    mint::Point2,
    Context, ContextBuilder, GameError, GameResult,
};

mod grid;
use grid::{GridCoord, SparseGrid};

const CELL_SIZE: (f32, f32) = (10.0, 10.0);
const GRID_SIZE: (usize, usize) = (100, 100);
const VIEW_SIZE: (usize, usize) = (200, 150);
const WINDOW_SIZE: (f32, f32) = (
    CELL_SIZE.0 * VIEW_SIZE.0 as f32,
    CELL_SIZE.1 * VIEW_SIZE.1 as f32,
);
const BG_COLOR: Color = Color::WHITE;
const CELL_COLOR: Color = Color::BLACK;
const LINE_WIDTH: f32 = 1.0;
const LINE_COLOR: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
const TEXT_COLOR: Color = Color::BLACK;

struct State {
    grid: SparseGrid,
    show_grid: bool,
    fps: u32,
    running: bool,
}

impl State {
    #[allow(unused)]
    pub fn new(ctx: &mut Context) -> Self {
        State {
            grid: SparseGrid::new(),
            show_grid: true,
            fps: 1,
            running: false,
        }
    }

    #[allow(unused)]
    pub fn rand(ctx: &mut Context) -> Self {
        let mut s = State::new(ctx);
        let off_x = (VIEW_SIZE.0 - GRID_SIZE.0) / 2;
        let off_y = (VIEW_SIZE.1 - GRID_SIZE.1) / 2;
        for x in off_x..(GRID_SIZE.0 + off_x) {
            for y in off_y..(GRID_SIZE.1 + off_y) {
                if rand::random() {
                    s.grid.set(GridCoord::Valid(x as i64, y as i64), 1);
                }
            }
        }

        s
    }
}

impl EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(self.fps) && self.running {
            let mut next = SparseGrid::new();
            for c in self.grid.elements() {
                next.tally(&c.expand());
            }

            next.retain(|gc, v| *v == 3 || (*v == 4 && self.grid.is_alive(gc)));

            self.grid = next;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, BG_COLOR);

        if self.show_grid {
            for i in 0..VIEW_SIZE.0 as usize {
                let line = Mesh::new_line(
                    ctx,
                    &vec![
                        Point2 {
                            x: i as f32 * CELL_SIZE.0,
                            y: 0.0,
                        },
                        Point2 {
                            x: i as f32 * CELL_SIZE.0,
                            y: WINDOW_SIZE.1,
                        },
                    ],
                    LINE_WIDTH,
                    LINE_COLOR,
                )?;
                canvas.draw(&line, graphics::DrawParam::from(Point2 { x: 0.0, y: 0.0 }));
            }

            for j in 0..VIEW_SIZE.1 as usize {
                let line = Mesh::new_line(
                    ctx,
                    &vec![
                        Point2 {
                            x: 0.0,
                            y: j as f32 * CELL_SIZE.1,
                        },
                        Point2 {
                            x: WINDOW_SIZE.0,
                            y: j as f32 * CELL_SIZE.1,
                        },
                    ],
                    LINE_WIDTH,
                    LINE_COLOR,
                )?;
                canvas.draw(&line, graphics::DrawParam::from(Point2 { x: 0.0, y: 0.0 }));
            }
        }

        for gc in self.grid.elements() {
            if let GridCoord::Valid(x, y) = gc {
                if x > 0 && x < VIEW_SIZE.0 as i64 && y > 0 && y < VIEW_SIZE.1 as i64 {
                    let rect = Mesh::new_rectangle(
                        ctx,
                        DrawMode::fill(),
                        Rect::new(
                            x as f32 * CELL_SIZE.0,
                            y as f32 * CELL_SIZE.1,
                            CELL_SIZE.0,
                            CELL_SIZE.1,
                        ),
                        CELL_COLOR,
                    )?;
                    canvas.draw(&rect, graphics::DrawParam::from(Point2 { x: 0.0, y: 0.0 }));
                }
            }
        }

        let text = Text::new(self.fps.to_string());
        canvas.draw(
            &text,
            graphics::DrawParam::from(Point2 { x: 0.0, y: 2.0 }).color(TEXT_COLOR),
        );

        canvas.finish(ctx)?;

        Ok(())
    }

    // fn mouse_button_down_event(
    //     &mut self,
    //     _ctx: &mut Context,
    //     _button: MouseButton,
    //     x: f32,
    //     y: f32,
    // ) -> GameResult {
    //     self.grid[(x / CELL_SIZE.0).floor() as usize][(y / CELL_SIZE.1).floor() as usize] ^= true;

    //     Ok(())
    // }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, repeat: bool) -> GameResult {
        if let Some(keycode) = input.keycode {
            if keycode == KeyCode::Space && !repeat {
                self.running ^= true;
            }
            if keycode == KeyCode::Up {
                self.fps += 1;
            }
            if keycode == KeyCode::Down && self.fps > 1 {
                self.fps -= 1;
            }
            if keycode == KeyCode::Delete {
                self.grid = SparseGrid::new();
            }
            if keycode == KeyCode::G {
                self.show_grid = !self.show_grid;
            }
        }

        Ok(())
    }
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("Conway's Game of Life", "mathletedev")
        .window_mode(WindowMode::default().dimensions(WINDOW_SIZE.0, WINDOW_SIZE.1))
        .build()?;

    let mut state = State::rand(&mut ctx);
    state.running = true;

    event::run(ctx, event_loop, state);
}
