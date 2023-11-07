use ggez::glam::*;
use ggez::graphics::PxScale;
use ggez::input::keyboard::{KeyCode, KeyInput, KeyMods};
use ggez::{
    conf::WindowMode,
    event::{self, EventHandler},
    graphics::{self, Color, DrawMode, Mesh, Rect, Text},
    mint::Point2,
    Context, ContextBuilder, GameError, GameResult,
};
use log::*;

mod grid;
use grid::{GridCoord, SparseGrid};

const DEFAULT_WINDOW_SIZE: (f32, f32) = (2000.0, 1500.0);

#[derive(Debug)]
struct ViewParams {
    window_size: (f32, f32),
    cell_size: (f32, f32),
    view_size: (usize, usize),
    grid_size: (usize, usize),
    line_width: f32,
}

impl ViewParams {
    fn resize(&mut self, x: f32, y: f32) {
        let vsx = x / self.cell_size.0;
        let vsy = y / self.cell_size.1;
        self.view_size = (vsx as usize, vsy as usize);
        self.window_size = (x, y);
    }
}

impl Default for ViewParams {
    fn default() -> Self {
        let mut vp = ViewParams {
            window_size: DEFAULT_WINDOW_SIZE,
            cell_size: (10.0, 10.0),
            view_size: (0, 0),
            grid_size: (100, 100),
            line_width: 1.0,
        };

        vp.resize(vp.window_size.0, vp.window_size.1);

        vp
    }
}

// const CELL_SIZE: (f32, f32) = (10.0, 10.0);
// const GRID_SIZE: (usize, usize) = (100, 100);
// const VIEW_SIZE: (usize, usize) = (200, 150);
// const WINDOW_SIZE: (f32, f32) = (
//     CELL_SIZE.0 * VIEW_SIZE.0 as f32,
//     CELL_SIZE.1 * VIEW_SIZE.1 as f32,
// );
// const LINE_WIDTH: f32 = 1.0;

const BG_COLOR: Color = Color::WHITE;
const CELL_COLOR: Color = Color::BLACK;
const LINE_COLOR: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
const TEXT_COLOR: Color = Color::BLACK;

struct State {
    view_params: ViewParams,
    grid: SparseGrid,
    show_grid: bool,
    fps: u32,
    running: bool,
    generation: u32,
    show_header: bool,
    actual_fps: f64,
    dirty: bool,
    cell_count: usize,
    xt: i64,
    yt: i64,
}

impl State {
    #[allow(unused)]
    pub fn new(ctx: &mut Context) -> Self {
        State {
            view_params: ViewParams::default(),
            grid: SparseGrid::new(),
            show_grid: true,
            fps: 1,
            running: false,
            generation: 0,
            show_header: true,
            actual_fps: 0.0,
            dirty: true,
            cell_count: 0,
            xt: 0,
            yt: 0,
        }
    }

    #[allow(unused)]
    pub fn rand(ctx: &mut Context) -> Self {
        let mut s = State::new(ctx);
        let view_size = s.view_params.view_size;
        let grid_size = s.view_params.grid_size;
        let off_x = (view_size.0 - grid_size.0) / 2;
        let off_y = (view_size.1 - grid_size.1) / 2;
        for x in off_x..(grid_size.0 + off_x) {
            for y in off_y..(grid_size.1 + off_y) {
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
        //debug!("Update called");
        while ctx.time.check_update_time(self.fps) && self.running {
            debug!("Update accepted...");

            self.generation += 1;
            let mut cell_count: usize = 0;
            let mut next = SparseGrid::new();
            for c in self.grid.elements() {
                next.tally(&c.expand());
                cell_count += 1;
            }

            next.retain(|gc, v| *v == 3 || (*v == 4 && self.grid.is_alive(gc)));

            // Note: this is the previous generation cell count... but should be good enough.
            //       To get this generation, we'd need a O(capacity) operation to count the retained keys.
            self.cell_count = cell_count;
            self.actual_fps = ctx.time.fps();
            self.dirty = true;

            self.grid = next;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        //debug!("Draw called");
        if !self.dirty {
            return Ok(());
        }

        //debug!("Dirty state to draw...");
        self.dirty = false;

        let mut canvas = graphics::Canvas::from_frame(ctx, BG_COLOR);

        let view_params = &self.view_params;

        if self.show_grid {
            for i in 0..view_params.view_size.0 as usize {
                let line = Mesh::new_line(
                    ctx,
                    &vec![
                        Point2 {
                            x: i as f32 * view_params.cell_size.0,
                            y: 0.0,
                        },
                        Point2 {
                            x: i as f32 * view_params.cell_size.0,
                            y: view_params.window_size.1,
                        },
                    ],
                    view_params.line_width,
                    LINE_COLOR,
                )?;
                canvas.draw(&line, graphics::DrawParam::from(Point2 { x: 0.0, y: 0.0 }));
            }

            for j in 0..view_params.view_size.1 as usize {
                let line = Mesh::new_line(
                    ctx,
                    &vec![
                        Point2 {
                            x: 0.0,
                            y: j as f32 * view_params.cell_size.1,
                        },
                        Point2 {
                            x: view_params.window_size.0,
                            y: j as f32 * view_params.cell_size.1,
                        },
                    ],
                    view_params.line_width,
                    LINE_COLOR,
                )?;
                canvas.draw(&line, graphics::DrawParam::from(Point2 { x: 0.0, y: 0.0 }));
            }
        }

        for gc in self.grid.elements() {
            if let GridCoord::Valid(x, y) = gc {
                let x = x + self.xt;
                let y = y + self.yt;
                if x >= 0
                    && x < view_params.view_size.0 as i64
                    && y >= 0
                    && y < view_params.view_size.1 as i64
                {
                    let rect = Mesh::new_rectangle(
                        ctx,
                        DrawMode::fill(),
                        Rect::new(
                            x as f32 * view_params.cell_size.0,
                            y as f32 * view_params.cell_size.1,
                            view_params.cell_size.0,
                            view_params.cell_size.1,
                        ),
                        CELL_COLOR,
                    )?;
                    canvas.draw(&rect, graphics::DrawParam::from(Point2 { x: 0.0, y: 0.0 }));
                }
            }
        }

        if self.show_header {
            let mut text = Text::new(format!(
                "FPS: {}, Pan: ({},{}), Generation: {}, Cells: {}",
                self.fps, self.xt, self.yt, self.generation, self.cell_count
            ));
            text.set_scale(PxScale::from(50.0));
            canvas.draw(
                &text,
                graphics::DrawParam::from(Point2 { x: 0.0, y: 2.0 }).color(TEXT_COLOR),
            );
        }

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
            let pan_delta = if input.mods.contains(KeyMods::SHIFT) {
                100
            } else {
                10
            };

            if keycode == KeyCode::Space && !repeat {
                self.running ^= true;
            }
            if keycode == KeyCode::Plus || keycode == KeyCode::Equals {
                self.fps += 1;
            }
            if (keycode == KeyCode::Minus || keycode == KeyCode::Underline) && self.fps > 1 {
                self.fps -= 1;
            }
            if keycode == KeyCode::Up {
                self.yt += pan_delta;
            }
            if keycode == KeyCode::Down {
                self.yt += -1 * pan_delta;
            }
            if keycode == KeyCode::Left {
                self.xt += pan_delta;
            }
            if keycode == KeyCode::Right {
                self.xt += -1 * pan_delta;
            }
            if keycode == KeyCode::C {
                self.xt = 0;
                self.yt = 0;
            }
            if keycode == KeyCode::Delete || keycode == KeyCode::Back {
                self.grid = SparseGrid::new();
                self.generation = 0;
            }
            if keycode == KeyCode::G {
                self.show_grid = !self.show_grid;
            }
            if keycode == KeyCode::H {
                self.show_header = !self.show_header;
            }
        }

        Ok(())
    }

    fn resize_event(
        &mut self,
        _ctx: &mut Context,
        width: f32,
        height: f32,
    ) -> Result<(), GameError> {
        info!("Resize: {}, {}", width, height);

        self.view_params.resize(width, height);
        Ok(())
    }
}

// Note: .env -> RUST_LOG=rusty_life=debug
fn main() -> GameResult {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default());

    let (mut ctx, event_loop) = ContextBuilder::new("Rusty Life", "Frank Taylor")
        .window_mode(
            WindowMode::default()
                .dimensions(DEFAULT_WINDOW_SIZE.0, DEFAULT_WINDOW_SIZE.1)
                .resizable(true),
        )
        .build()?;

    let mut state = State::rand(&mut ctx);
    state.running = true;

    event::run(ctx, event_loop, state);
}
