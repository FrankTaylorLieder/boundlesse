use ggez::glam::*;
use ggez::input::keyboard::KeyCode;
use ggez::input::keyboard::KeyInput;
use ggez::{
    conf::WindowMode,
    event::{self, EventHandler, MouseButton},
    graphics::{self, Color, DrawMode, Mesh, Rect, Text},
    mint::Point2,
    timer, Context, ContextBuilder, GameError, GameResult,
};

const CELL_SIZE: (f32, f32) = (20.0, 20.0);
const GRID_SIZE: (usize, usize) = (40, 40);
const WINDOW_SIZE: (f32, f32) = (
    CELL_SIZE.0 * GRID_SIZE.0 as f32,
    CELL_SIZE.1 * GRID_SIZE.1 as f32,
);
const BG_COLOR: Color = Color::WHITE;
const CELL_COLOR: Color = Color::BLACK;
const LINE_WIDTH: f32 = 2.0;
const LINE_COLOR: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
const TEXT_COLOR: Color = Color::BLACK;

struct State {
    screen: graphics::ScreenImage,
    grid: Vec<Vec<bool>>,
    fps: u32,
    running: bool,
}

impl State {
    #[allow(unused)]
    pub fn new(ctx: &mut Context) -> Self {
        State {
            screen: graphics::ScreenImage::new(
                ctx,
                graphics::ImageFormat::Rgba8UnormSrgb,
                1.,
                1.,
                1,
            ),
            grid: vec![vec![false; GRID_SIZE.1 as usize]; GRID_SIZE.0 as usize],
            fps: 1,
            running: false,
        }
    }

    #[allow(unused)]
    pub fn rand(ctx: &mut Context) -> Self {
        let mut s = State::new(ctx);
        for x in 0..GRID_SIZE.0 {
            for y in 0..GRID_SIZE.1 {
                s.grid[x][y] = rand::random();
            }
        }

        s
    }
}

impl EventHandler<GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(self.fps) && self.running {
            let mut coords: Vec<(usize, usize)> = vec![];

            for i in 0..GRID_SIZE.0 as usize {
                let left = if i > 0 {
                    i - 1
                } else {
                    GRID_SIZE.0 as usize - 1
                };
                let right = if i < GRID_SIZE.0 as usize - 1 {
                    i + 1
                } else {
                    0
                };
                for j in 0..GRID_SIZE.1 as usize {
                    let up = if j > 0 {
                        j - 1
                    } else {
                        GRID_SIZE.1 as usize - 1
                    };
                    let down = if j < GRID_SIZE.1 as usize - 1 {
                        j + 1
                    } else {
                        0
                    };

                    let neighbors = self.grid[left][j] as u8
                        + self.grid[left][up] as u8
                        + self.grid[i][up] as u8
                        + self.grid[right][up] as u8
                        + self.grid[right][j] as u8
                        + self.grid[right][down] as u8
                        + self.grid[i][down] as u8
                        + self.grid[left][down] as u8;

                    if (self.grid[i][j] && (neighbors < 2 || neighbors > 3))
                        || (!self.grid[i][j] && neighbors == 3)
                    {
                        coords.push((i, j));
                    }
                }
            }

            for coord in coords {
                self.grid[coord.0][coord.1] ^= true;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        //graphics::clear(ctx, BG_COLOR);
        let mut canvas = graphics::Canvas::from_frame(ctx, BG_COLOR);
        //let mut canvas = graphics::Canvas::from_screen_image(ctx, &mut self.screen, BG_COLOR);

        for i in 0..GRID_SIZE.0 as usize {
            for j in 0..GRID_SIZE.1 as usize {
                if self.grid[i][j] {
                    let rect = Mesh::new_rectangle(
                        ctx,
                        DrawMode::fill(),
                        Rect::new(
                            i as f32 * CELL_SIZE.0,
                            j as f32 * CELL_SIZE.1,
                            CELL_SIZE.0,
                            CELL_SIZE.1,
                        ),
                        CELL_COLOR,
                    )?;
                    canvas.draw(&rect, graphics::DrawParam::from(Point2 { x: 0.0, y: 0.0 }));
                }

                if j == 0 {
                    continue;
                }

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

            if i == 0 {
                continue;
            }

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

        let text = Text::new(self.fps.to_string());
        canvas.draw(
            &text,
            graphics::DrawParam::from(Point2 { x: 0.0, y: 2.0 }).color(TEXT_COLOR),
        );

        canvas.finish(ctx)?;

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.grid[(x / CELL_SIZE.0).floor() as usize][(y / CELL_SIZE.1).floor() as usize] ^= true;

        Ok(())
    }

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
                self.grid = vec![vec![false; GRID_SIZE.1 as usize]; GRID_SIZE.0 as usize];
            }
        }

        Ok(())
    }
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("Conway's Game of Life", "mathletedev")
        .window_mode(WindowMode::default().dimensions(WINDOW_SIZE.0, WINDOW_SIZE.1))
        .build()?;

    let state = State::rand(&mut ctx);

    event::run(ctx, event_loop, state);
}
