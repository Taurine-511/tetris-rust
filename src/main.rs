use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{
    collections::VecDeque,
    io::{self, Write},
    time::{Duration, Instant},
    thread,
};
use crossterm::execute;

type Grid = Vec<Vec<bool>>;

#[derive(Clone)]
pub struct Field {
    grid: Grid,
}

#[derive(Clone, Copy)]
pub enum BlockShape {
    I,
    O,
    S,
    Z,
    J,
    L,
    T,
}

impl Distribution<BlockShape> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BlockShape {
        use BlockShape::*;
        match rng.gen_range(0..7) {
            0 => I,
            1 => O,
            2 => S,
            3 => Z,
            4 => J,
            5 => L,
            _ => T,
        }
    }
}

pub struct Block {
    coord: (i8, i8),
    coord_max: (i8, i8),
    rotation: usize,
    shapes: Vec<Grid>,
}

impl Block {
    pub fn new(shape_type: BlockShape, coord_max: (i8, i8)) -> Self {
        let base_shape = Block::get_base_shape(shape_type);
        let shapes = Block::generate_rotations(base_shape);
        Self {
            coord: (4, 0),
            rotation: 0,
            shapes,
            coord_max,
        }
    }

    fn get_base_shape(shape_type: BlockShape) -> Vec<Vec<u8>> {
        match shape_type {
            BlockShape::I => vec![
                vec![0, 0, 0, 0],
                vec![1, 1, 1, 1],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
            ],
            BlockShape::O => vec![vec![1, 1], vec![1, 1]],
            BlockShape::S => vec![vec![0, 1, 1], vec![1, 1, 0], vec![0, 0, 0]],
            BlockShape::Z => vec![vec![1, 1, 0], vec![0, 1, 1], vec![0, 0, 0]],
            BlockShape::J => vec![vec![1, 0, 0], vec![1, 1, 1], vec![0, 0, 0]],
            BlockShape::L => vec![vec![0, 0, 1], vec![1, 1, 1], vec![0, 0, 0]],
            BlockShape::T => vec![vec![0, 1, 0], vec![1, 1, 1], vec![0, 0, 0]],
        }
    }

    fn convert_to_bool(grid: Vec<Vec<u8>>) -> Grid {
        grid.into_iter()
            .map(|row| row.into_iter().map(|cell| cell == 1).collect())
            .collect()
    }

    fn rotate_90(grid: &Grid) -> Grid {
        let rows = grid.len();
        let cols = grid[0].len();
        let mut new_grid = vec![vec![false; rows]; cols];

        for i in 0..rows {
            for j in 0..cols {
                new_grid[j][rows - 1 - i] = grid[i][j];
            }
        }
        new_grid
    }

    fn generate_rotations(base_shape: Vec<Vec<u8>>) -> Vec<Grid> {
        let shape = Block::convert_to_bool(base_shape);
        let mut rotations = vec![shape.clone()];
        let mut current_shape = shape;

        for _ in 0..3 {
            current_shape = Block::rotate_90(&current_shape);
            rotations.push(current_shape.clone());
        }
        rotations
    }

    pub fn rotate_left(&mut self) {
        self.rotation = (self.rotation + 3) % 4;
    }

    pub fn rotate_right(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    pub fn down(&mut self) {
        self.coord.1 = std::cmp::min(self.coord.1 + 1, self.coord_max.1);
    }

    pub fn left(&mut self) {
        self.coord.0 = std::cmp::max(self.coord.0 - 1, -2);
    }

    pub fn right(&mut self) {
        self.coord.0 = std::cmp::min(self.coord.0 + 1, self.coord_max.0);
    }
}

impl Field {
    pub fn new(width: usize, height: usize) -> Self {
        let mut grid = vec![vec![false; width + 2]; height + 1];
        grid[height].fill(true);
        for y in 0..height {
            grid[y][0] = true;
            grid[y][width + 1] = true;
        }
        Self { grid }
    }

    pub fn set(&mut self, grid: Grid) {
        self.grid = grid;
    }

    pub fn arrange_with_block(&self, block: &Block) -> Option<Grid> {
        let mut new_grid = self.grid.clone();
        for (dy, row) in block.shapes[block.rotation].iter().enumerate() {
            for (dx, &cell) in row.iter().enumerate() {
                if cell {
                    let x = block.coord.0 + dx as i8;
                    let y = block.coord.1 + dy as i8;
                    if y as usize >= new_grid.len() || x as usize >= new_grid[0].len() {
                        return None;
                    }
                    if new_grid[y as usize][x as usize] {
                        return None;
                    }
                    new_grid[y as usize][x as usize] = true;
                }
            }
        }
        Some(new_grid)
    }

    pub fn update(&mut self) {
        let mut queue: VecDeque<usize> = VecDeque::new();
        for y in (0..self.grid.len() - 1).rev() {
            if self.is_row_full(y) {
                queue.push_back(y);
                self.clear_row(y);
            } else if let Some(deleted) = queue.pop_front() {
                self.grid.swap(deleted, y);
                queue.push_back(y);
            }
        }
    }

    fn is_row_full(&self, y: usize) -> bool {
        self.grid[y].iter().all(|&cell| cell)
    }

    fn clear_row(&mut self, y: usize) {
        self.grid[y].fill(false);
        self.grid[y][0] = true;
        let n = self.grid[y].len() - 1;
        self.grid[y][n] = true;
    }

    fn format_grid(grid: &Grid) -> String {
        grid.iter()
            .map(|row| {
                row.iter()
                    .map(|&cell| if cell { "[]" } else { "  " })
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\r\n")
    }
}

enum Operation {
    Right,
    Left,
    Down,
    LRot,
    RRot,
}

pub struct Game {
    field: Field,
    block: Option<Block>,
    last_drop_time: Instant,
    drop_interval: Duration,
}

impl Game {
    pub fn new(width: usize, height: usize, drop_interval: Duration) -> Self {
        Self {
            field: Field::new(width, height),
            block: None,
            last_drop_time: Instant::now(),
            drop_interval,
        }
    }

    pub fn new_block(&mut self) {
        let shape: BlockShape = rand::random();
        let coord_max = (
            self.field.grid[0].len() as i8,
            self.field.grid.len() as i8,
        );
        self.block = Some(Block::new(shape, coord_max));
    }

    fn parse_input(input: Option<char>) -> Option<Operation> {
        input.and_then(|key| match key {
            'a' => Some(Operation::Left),
            'd' => Some(Operation::Right),
            's' => Some(Operation::Down),
            'q' => Some(Operation::LRot),
            'e' => Some(Operation::RRot),
            _ => None,
        })
    }

    fn apply_operation(block: &mut Block, operation: Operation) {
        match operation {
            Operation::Left => block.left(),
            Operation::Right => block.right(),
            Operation::Down => block.down(),
            Operation::LRot => block.rotate_left(),
            Operation::RRot => block.rotate_right(),
        }
    }

    pub fn step(&mut self, input: Option<char>) {
        if let Some(ref mut block) = self.block {
            if let Some(operation) = Self::parse_input(input) {
                let prev_rot = block.rotation.clone();
                let prev_coord = block.coord.clone();
                Self::apply_operation(block, operation);
                if let Some(grid_with_block) = self.field.arrange_with_block(&block) {
                    Self::render(Field::format_grid(&grid_with_block));
                } else {
                    block.rotation = prev_rot;
                    block.coord = prev_coord;
                }
            }
        }
    }

    pub fn drop(&mut self) {
        let mut formatted = Field::format_grid(&self.field.grid);
        if let Some(ref mut block) = self.block {
            let prev_rot = block.rotation.clone();
            let prev_coord = block.coord.clone();
            block.down();
            if let Some(grid_with_block) = self.field.arrange_with_block(block) {
                formatted = Field::format_grid(&grid_with_block);
            } else {
                block.rotation = prev_rot;
                block.coord = prev_coord;
                self.update_state();
            }
        }
        Self::render(formatted);
    }

    fn update_state(&mut self) {
        if let Some(block) = &self.block {
            if let Some(grid_with_block) = self.field.arrange_with_block(block) {
                self.field.set(grid_with_block);
                self.field.update();
            }
        }
        self.new_block();
    }

    pub fn update(&mut self, input: Option<char>) {
        let now = Instant::now();

        // Process user input if any
        if input.is_some() {
            self.step(input);
        }

        // Drop block if enough time has passed
        if now.duration_since(self.last_drop_time) >= self.drop_interval {
            self.drop();
            self.last_drop_time = now;
        }
    }

    fn render(formatted: String) {
        execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All), crossterm::cursor::MoveTo(0, 0)).unwrap();
        print!("{}", formatted);
        io::stdout().flush().unwrap();
    }
}

const CLOCK_TIME: Duration = Duration::from_millis(500);

fn main() -> io::Result<()> {
    use crossterm::event::{self, Event, KeyCode};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

    enable_raw_mode()?;
    let mut game = Game::new(10, 22, CLOCK_TIME);
    game.new_block();

    loop {
        // Non-blocking user input check
        let input = if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('p') => break,
                    KeyCode::Char(c) => Some(c),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        game.update(input);
        thread::sleep(Duration::from_millis(16)); // Approx 60 FPS
    }
    
    disable_raw_mode()?;
    Ok(())
}
