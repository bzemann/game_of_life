use error_iter::ErrorIter as _;
use log::{debug, error};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::event::VirtualKeyCode;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
        WindowBuilder::new()
            .with_title("Conway's Game of Life")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut game = GameOfLife::new(WIDTH as usize, HEIGHT as usize);
    let mut paused = false;

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            game.draw(pixels.frame_mut());
            if let Err(e) = pixels.render() {
                error!("pixels.render() failed: {}", e);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.key_pressed(VirtualKeyCode::P) {
                paused = !paused;
            }
            if input.key_pressed_os(VirtualKeyCode::Space) {
                paused = true;
            }
            if input.key_pressed(VirtualKeyCode::R) {
                game.starting_position();
            }
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            if !paused || input.key_pressed_os(VirtualKeyCode::Space) {
                game.update();
            }
            window.request_redraw();
        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

struct GameOfLife {
    width: usize,
    height: usize,
    cells: Vec<Vec<bool>>, // true = alive, false = dea
}

impl GameOfLife {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![vec![false; width]; height],
        }
    }

    fn update(&mut self) {
        let new_cells = (0..self.height)
            .map(|row| {
                (0..self.width)
                    .map(|col| self.compute_next_states(row, col))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        self.cells = new_cells;
    }

    fn compute_next_states(&self, row: usize, col: usize) -> bool {
        let alive_neighbors = self.count_neighbors(row, col);
        match (self.cells[row][col], alive_neighbors) {
            (true, 2) | (true, 3) => true,
            (false, 3) => true,
            _ => false,
        }
    }

    fn count_neighbors(&self, row: usize, col: usize) -> usize {
        let mut count = 0;
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let row_neighbor =
                    (row as isize + dr + self.height as isize) as usize % self.height;
                let col_neighbor = (col as isize + dc + self.width as isize) as usize % self.width;

                if self.cells[row_neighbor][col_neighbor] {
                    count += 1;
                }
            }
        }
        count
    }

    fn draw(&self, screen: &mut [u8]) {
        for row in 0..self.height {
            for col in 0..self.width {
                let index = (row * self.width + col) * 4;
                let color = if self.cells[row][col] {
                    [0x00, 0x00, 0x00]
                } else {
                    [0xFF, 0xFF, 0xFF]
                };
                screen[index] = color[0];
                screen[index + 1] = color[1];
                screen[index + 2] = color[2];
                screen[index + 3] = 0xFF;
            }
        }
    }

    fn draw_terminal(&self) {
        for row in 0..self.height {
            for col in 0..self.width {
                let symbol = if self.cells[row][col] {
                    "◼︎"
                } else {
                    "◻︎"
                };
                print!("{}", symbol);
            }
            println!();
        }
    }

    fn simulate(&mut self, steps: usize) {
        self.draw_terminal();
        for _ in 0..steps {
            self.update();
            println!("{}[2J", 27 as char);
            self.draw_terminal();
        }
    }

    fn starting_position(&mut self) {
        let middle_row = self.height / 2;
        let middle_col = self.width / 2;

        let offsets = [(-1, -1), (-1, 0), (0, -2), (0, -1), (1, -1)];

        for &(dr, dc) in offsets.iter() {
            let row = (middle_row as isize + dr) as usize;
            let col = (middle_col as isize + dc) as usize;

            if row < self.height && col < self.width {
                self.cells[row][col] = true;
            }
        }
    }
}
