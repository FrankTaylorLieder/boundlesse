#[allow(unused_imports)]
use ggez::glam::*;
use ggez::graphics::{MeshBuilder, PxScale};
use ggez::input::keyboard::{KeyCode, KeyInput, KeyMods};
use ggez::{
    conf::WindowMode,
    event::{self, EventHandler},
    graphics::{self, Color, DrawMode, DrawParam, Mesh, Rect, Text},
    mint::Point2,
    Context, ContextBuilder, GameError, GameResult,
};
use log::*;
use std::time::SystemTime;
use std::{env, thread};

mod grid;
use grid::{GridCoord, Universe};
use rle::{load_rle, Inject};

mod rle;

fn now() -> u128 {
    let duration_since_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    duration_since_epoch.as_nanos() / 1000
}

// Don't start additional udpates in Update() if we've spent more than this time (1000th ms) here already.
const LIVENESS_TARGET: u128 = 100 * 1000;
const DEFAULT_WINDOW_SIZE: (f32, f32) = (2000.0, 1500.0);

#[derive(Debug)]
struct ViewParams {
    // Reported window size in pixels.
    window_size: (f32, f32),

    // Cell size in pixels.
    cell_size: f32,

    // Visible area size in cells.
    view_size: (i64, i64),

    // Size of pattern (in cells) to create when random data is requested.
    pattern_size: (i64, i64),

    // Viewport panning offset in cells.
    xt: i64,
    yt: i64,
}

impl ViewParams {
    fn resize_aux(&mut self) {
        let old_view_size = self.view_size;

        let vsx = self.window_size.0 / self.cell_size + 1.0;
        let vsy = self.window_size.1 / self.cell_size + 1.0;
        self.view_size = (vsx as i64, vsy as i64);

        let ((ovx, ovy), (nvx, nvy)) = (old_view_size, self.view_size);
        let view_deltas: (i64, i64) = (nvx - ovx, nvy - ovy);

        self.xt += view_deltas.0 / 2;
        self.yt += view_deltas.1 / 2;
    }

    fn resize_window(&mut self, x: f32, y: f32) {
        self.window_size = (x, y);
        self.resize_aux();
    }

    fn resize_zoom(&mut self) {
        self.resize_aux();
    }
}

impl Default for ViewParams {
    fn default() -> Self {
        let cs = 10.0;
        let vs = (
            (DEFAULT_WINDOW_SIZE.0 / cs) as i64,
            (DEFAULT_WINDOW_SIZE.1 / cs) as i64,
        );
        let mut vp = ViewParams {
            window_size: DEFAULT_WINDOW_SIZE,
            cell_size: cs,
            view_size: vs,
            pattern_size: (200, 200),
            xt: vs.0 / 2,
            yt: vs.1 / 2,
        };

        vp.resize_window(vp.window_size.0, vp.window_size.1);

        vp
    }
}

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
    universe: Universe,
    show_grid: bool,
    gps: u32,
    running: bool,
    show_header: bool,
    actual_fps: f64,
    dirty: bool,
    cell_count: usize,
    updates: u32,
    draws: u32,
}

impl State {
    pub fn new(_ctx: &mut Context) -> Self {
        State {
            view_params: ViewParams::default(),
            universe: Universe::new(),
            show_grid: true,
            gps: 10,
            running: false,
            show_header: true,
            actual_fps: 0.0,
            dirty: true,
            cell_count: 0,
            updates: 0,
            draws: 0,
        }
    }

    pub fn seed_rand(&mut self) {
        let view_size = self.view_params.view_size;
        let grid_size = self.view_params.pattern_size;
        let off_x = ((view_size.0 - grid_size.0) / 2) - self.view_params.xt;
        let off_y = ((view_size.1 - grid_size.1) / 2) - self.view_params.yt;
        for x in off_x..(grid_size.0 + off_x) {
            for y in off_y..(grid_size.1 + off_y) {
                if rand::random() {
                    self.universe.grid.set(GridCoord::Valid(x, y));
                }
            }
        }
    }

    pub fn load_rle(&mut self, filename: &str) -> GameResult {
        let mut injector = Injector::new(self);
        load_rle(filename, &mut injector, true)
            .map_err(|e| GameError::CustomError(e.to_string()))?;

        info!("Loaded pattern: {} cells", injector.cells);
        Ok(())
    }
}

pub struct Injector<'a> {
    state: &'a mut State,
    cells: usize,
}

impl<'a> Injector<'a> {
    fn new(state: &'a mut State) -> Self {
        Self { state, cells: 0 }
    }
}

impl<'a> Inject for Injector<'a> {
    fn inject(&mut self, coord: GridCoord, alive: bool) -> anyhow::Result<()> {
        if alive {
            self.state.universe.grid.set(coord);
            trace!("Setting coord: {:?}", coord);
            self.cells += 1;
        } else {
            self.state.universe.grid.unset(coord);
            trace!("Unsetting coord: {:?}", coord);
        }
        Ok(())
    }
}

impl EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        trace!("Update requested...");

        let start = now();
        while ctx.time.check_update_time(self.gps)
            && self.running
            && (now() - start < LIVENESS_TARGET)
        {
            trace!("Update accepted...{}", self.universe.generation);
            let us = now();

            self.updates += 1;
            if self.updates % self.gps == 0 {
                debug!("Updates: {}", self.updates);
            }

            let cell_count = self.universe.update();

            // Note: this is the previous generation cell count... but should be good enough.
            //       To get this generation, we'd need a O(capacity) operation to count the retained keys.
            self.cell_count = cell_count;
            self.actual_fps = ctx.time.fps();
            self.dirty = true;

            let ds = now() - us;
            trace!("Update done: {ds} - {}", self.cell_count);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        trace!("Draw requested...");

        self.draws += 1;
        if self.draws % 60 == 0 {
            debug!("Draws: {}", self.draws);
        }

        if !self.dirty {
            trace!("Draw rejected");
            return Ok(());
        }

        trace!("Draw accepted...");
        let start = now();

        self.dirty = false;

        let mut canvas = graphics::Canvas::from_frame(ctx, BG_COLOR);

        let view_params = &self.view_params;

        if self.show_grid {
            let mut lb = MeshBuilder::new();
            for i in 0..view_params.view_size.0 as usize {
                lb.line(
                    &[
                        vec2(i as f32 * view_params.cell_size, 0.0),
                        vec2(i as f32 * view_params.cell_size, view_params.window_size.1),
                    ],
                    1.0,
                    LINE_COLOR,
                )?;
            }

            for j in 0..view_params.view_size.1 as usize {
                lb.line(
                    &[
                        vec2(0.0, j as f32 * view_params.cell_size),
                        vec2(view_params.window_size.0, j as f32 * view_params.cell_size),
                    ],
                    1.0,
                    LINE_COLOR,
                )?;
            }

            let mesh = lb.build();
            canvas.draw(&Mesh::from_data(ctx, mesh), DrawParam::default());
        }

        let mut cells_drawn = 0;
        let mut cb = MeshBuilder::new();
        for gc in self.universe.grid.live_cells() {
            if let GridCoord::Valid(x, y) = gc {
                let x = x + self.view_params.xt;
                let y = y + self.view_params.yt;
                if x >= 0 && x < view_params.view_size.0 && y >= 0 && y < view_params.view_size.1 {
                    cells_drawn += 1;
                    let cs = view_params.cell_size;
                    cb.rectangle(
                        DrawMode::fill(),
                        Rect::new(x as f32 * cs, y as f32 * cs, cs, cs),
                        CELL_COLOR,
                    )?;
                }
            }
        }

        canvas.draw(&Mesh::from_data(ctx, cb.build()), DrawParam::default());

        trace!("Draw finished: {} took {}", cells_drawn, now() - start);

        if self.show_header {
            let mut text = Text::new(format!(
                "{}, GPS: {}, FPS: {:.2}, Pan: ({},{}), Cell size: {}, Generation: {}, Cells: {}",
                if self.running { "Running" } else { "Stopped" },
                self.gps,
                self.actual_fps,
                self.view_params.xt,
                self.view_params.yt,
                self.view_params.cell_size,
                self.universe.generation,
                self.cell_count
            ));
            text.set_scale(PxScale::from(40.0));
            canvas.draw(
                &text,
                graphics::DrawParam::from(Point2 { x: 0.0, y: 2.0 }).color(TEXT_COLOR),
            );
        }

        canvas.finish(ctx)?;

        let duration = now() - start;
        trace!("Draw done: {duration}");

        thread::yield_now();

        Ok(())
    }

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
                self.gps += u32::max(self.gps / 10, 1);
            }
            if (keycode == KeyCode::Minus || keycode == KeyCode::Underline) && self.gps > 1 {
                self.gps -= u32::max(self.gps / 10, 1);
            }
            if keycode == KeyCode::Up {
                self.view_params.yt += pan_delta;
            }
            if keycode == KeyCode::Down {
                self.view_params.yt += -pan_delta;
            }
            if keycode == KeyCode::Left {
                self.view_params.xt += pan_delta;
            }
            if keycode == KeyCode::Right {
                self.view_params.xt += -pan_delta;
            }
            if keycode == KeyCode::C {
                self.view_params.xt = self.view_params.view_size.0 / 2;
                self.view_params.yt = self.view_params.view_size.1 / 2;
            }
            if keycode == KeyCode::Delete || keycode == KeyCode::Back {
                self.universe = Universe::new();
                self.cell_count = 0;
            }
            if keycode == KeyCode::G {
                self.show_grid = !self.show_grid;
            }
            if keycode == KeyCode::H {
                self.show_header = !self.show_header;
            }
            if keycode == KeyCode::A && self.view_params.cell_size > 1.0 {
                self.view_params.cell_size -= 1.0;
                self.view_params.resize_zoom();
            }
            if keycode == KeyCode::S {
                self.view_params.cell_size += 1.0;
                self.view_params.resize_zoom();
            }
            if keycode == KeyCode::R {
                self.seed_rand();
            }

            self.dirty = true;
        }

        Ok(())
    }

    fn resize_event(
        &mut self,
        _ctx: &mut Context,
        width: f32,
        height: f32,
    ) -> Result<(), GameError> {
        debug!("Resize: {}, {}", width, height);

        self.view_params.resize_window(width, height);
        self.dirty = true;
        Ok(())
    }
}

// Note: .env -> RUST_LOG=boundless=debug
fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default());

    info!("Starting rusty-life...");

    let args: Vec<String> = env::args().collect();

    let (mut ctx, event_loop) = ContextBuilder::new("Rusty Life", "Frank Taylor")
        .window_mode(
            WindowMode::default()
                .dimensions(DEFAULT_WINDOW_SIZE.0, DEFAULT_WINDOW_SIZE.1)
                .resizable(true),
        )
        .build()?;

    let mut state = State::new(&mut ctx);
    if args.len() > 1 {
        info!("Loading pattern: {}", args[1]);
        state.load_rle(args[1].as_str())?;
    }

    state.running = false;

    event::run(ctx, event_loop, state);
}
