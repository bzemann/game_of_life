use pixels::{Pixels, SurfaceTexture};
use std::error::Error;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
const WIDTH_WINDOW: usize = 1920;
const HEIGHT_WINDOW: usize = 1080;
const WIDTH_GRID: usize = WIDTH_WINDOW / CELL_SIZE;
const HEIGHT_GRID: usize = HEIGHT_WINDOW / CELL_SIZE;
const PIXEL_SIZE: usize = 4;
const CELL_SIZE: usize = 10;
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

    fn render(&self, frame: &mut [u8]) {
        for (y, row) in self.cells.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                for dy in 0..CELL_SIZE {
                    for dx in 0..CELL_SIZE {
                        let pixel_x = (x * CELL_SIZE + dx) * PIXEL_SIZE;
                        let pixel_y = (y * CELL_SIZE + dy) * WIDTH_WINDOW * PIXEL_SIZE;
                        let pixel_index = pixel_y + pixel_x;

                        if pixel_index + PIXEL_SIZE <= frame.len() {
                            let color = if cell {
                                [0x00, 0x00, 0x00, 0xff] // Black
                            } else {
                                [0xff, 0xff, 0xff, 0xff] // White
                            };
                            frame[pixel_index..pixel_index + PIXEL_SIZE].copy_from_slice(&color);
                        }
                    }
                }
            }
        }
    }

    fn toggle_state(&mut self, row: usize, col: usize) {
        self.cells[row][col] = !self.cells[row][col];
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Conway's Game of Life")
        .with_inner_size(LogicalSize::new(WIDTH_WINDOW as f32, HEIGHT_WINDOW as f32))
        .build(&event_loop)?;

    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(WIDTH_WINDOW as u32, HEIGHT_WINDOW as u32, surface_texture)?;

    let mut grid = GameOfLife::new(WIDTH_GRID, HEIGHT_GRID);
    grid.starting_position();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                grid.update();
                grid.render(pixels.get_frame_mut());
                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }

        // Request the next frame.
        window.request_redraw();
    });
}
