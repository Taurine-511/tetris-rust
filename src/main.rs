use std::{collections::VecDeque, io::{self, Write}, time::Duration};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

type Grid = Vec<Vec<bool>>; 
pub struct Field {
    field: Grid,
}



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
    rotate: usize,
    shape: Vec<Grid>,
}

impl Block {
    pub fn rotate_left(&mut self) { 
        self.rotate += 3;
        self.rotate %= 4;
    }

    pub fn rotate_right(&mut self) {
        self.rotate += 1;
        self.rotate %= 4;
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

    pub fn new(shape_type: BlockShape, coord_max: (i8, i8)) -> Self {
        let shape_0_1 = match shape_type {
            BlockShape::I => vec![
                vec![0, 0, 0, 0],
                vec![1, 1, 1, 1],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
            ],
            BlockShape::O => vec![
                vec![1, 1],
                vec![1, 1],
            ],
            BlockShape::S => vec![
                vec![0, 1, 1],
                vec![1, 1, 0],
                vec![0, 0, 0],
            ],
            BlockShape::Z => vec![
                vec![1, 1, 0],
                vec![0, 1, 1],
                vec![0, 0, 0],
            ],
            BlockShape::J => vec![
                vec![1, 0, 0],
                vec![1, 1, 1],
                vec![0, 0, 0],
            ],
            BlockShape::L => vec![
                vec![0, 0, 1],
                vec![1, 1, 1],
                vec![0, 0, 0],
            ],
            BlockShape::T => vec![
                vec![0, 1, 0],
                vec![1, 1, 1],
                vec![0, 0, 0],
            ],
        };
        let shape = Block::generate_rotations(shape_0_1);
        Self {
            coord: (4, 0),
            rotate: 0,
            shape,
            coord_max,
        }
    }

    fn convert_to_bool(shape: Vec<Vec<u8>>) -> Grid {
        shape
            .into_iter().map(|row|
            row.into_iter().map(|elem| elem == 1).collect()
            ).collect()
    }

    fn rotate_90(shape: &Grid) -> Grid {
        let rows = shape.len();
        let cols = shape[0].len();
        let mut new_shape = vec![vec![false; rows]; cols];

        for i in 0..rows {
            for j in 0..cols {
                new_shape[j][rows - 1 - i] = shape[i][j];
            }
        }
        new_shape
    }

    fn generate_rotations(shape_0_1: Vec<Vec<u8>>) -> Vec<Grid> {
        let mut rotations = Vec::new();
        let shape_0 = Block::convert_to_bool(shape_0_1);
        let shape_90 = Block::rotate_90(&shape_0);
        let shape_180 = Block::rotate_90(&shape_90);
        let shape_270 = Block::rotate_90(&shape_180);

        rotations.push(shape_0);
        rotations.push(shape_90);
        rotations.push(shape_180);
        rotations.push(shape_270);

        rotations
    }
}

impl Field {
    pub fn new(width: usize, height: usize) -> Self {
        let mut field = vec![vec![false; width + 2]; height + 1];
        field[height].fill(true);
        for y in 0..height {
            field[y][0] = true;
            field[y][width + 1] = true;
        }

        Self {
            field,
        }
    }

    pub fn set(&mut self, field: Grid){
        self.field = field;
    }

    pub fn init_with_str(formatted: &str) -> Vec<Vec<bool>> {
        formatted.lines().fold(Vec::new(), |mut v ,w_str| {
            if w_str.len() != 0 {
                v.push(
                    w_str.chars()
                        .collect::<Vec<char>>()
                        .chunks(2)
                        .map(|chunk| chunk == ['[', ']'])
                        .collect::<Vec<bool>>()
                )
            }
            v
        })
    }

    pub fn arrange_with_block(&self, block: &Block) -> Option<Grid> {
        let mut output = self.field.clone();
        let cols_block = block.shape[0].len();
        let rows_block = block.shape[0][0].len();
        for y in 0..cols_block {
            for x in 0..rows_block {
                let y_check = (block.coord.1 + y as i8) as usize;
                let x_check = (block.coord.0 + x as i8) as usize;
                if y_check >= output.len() || x_check >= output[0].len() {
                    continue;
                }
                let ref mut elem = output[y_check][x_check];
                let block_elem = block.shape[block.rotate][y][x];
                if *elem && block_elem {
                    return None;
                }
                *elem |= block_elem;
            }
        }
        Some(output)
    }

    pub fn update(&mut self) {
        let mut queue: VecDeque<usize> = VecDeque::new();
        let field_len = self.field.len() - 1;
        for y in (0..field_len).rev() {
            if self.is_row_full(y) {
                queue.push_back(y);
                self.clear_row(y);
            } else if let Some(deleted) = queue.pop_front() {
                self.field.swap(deleted, y);
                queue.push_back(y);
            }
        }
    }

    fn is_row_full(&self, y: usize) -> bool {
        self.field[y].iter().all(|&elem| elem)
    }

    fn clear_row(&mut self, y: usize) {
        let row = &mut self.field[y];
        let n = row.len();
        row.fill(false);
        row[0] = true;
        row[n - 1] = true;
    }


    fn format_field(field: &Grid) -> String {
        field.iter().map(|w_vec| {
            w_vec.iter().map(|elem| {
                if *elem { "[]" } else { "  " }
            }).collect::<String>()
        }).collect::<Vec<String>>().join("\r\n")
    }
}

enum Operation{
    Right,
    Left,
    Down,
    LRot,
    RRot,
}

pub struct Game {
    field: Field,
    block: Option<Block>,
    last_drop_time: std::time::Instant,
    drop_interval: std::time::Duration,
}

impl Game {
    pub fn new(width: usize, height: usize, drop_interval: Duration) -> Self {
        Self {
            field: Field::new(width, height),
            block: None,            
            last_drop_time: std::time::Instant::now(),
            drop_interval,
        }
    }

    pub fn new_block(&mut self) {
        let shape: BlockShape = rand::random();
        let coord_max = (self.field.field[0].len() as i8 , self.field.field.len() as i8);
        self.block = Some(Block::new(shape, coord_max));
    }

    fn parse(input: Option<char>) -> Option<Operation> {
        let key = input?;
        let op = match key {
            'a' => Some(Operation::Left),
            'd' => Some(Operation::Right),
            's' => Some(Operation::Down),
            'q' => Some(Operation::LRot),
            'e' => Some(Operation::RRot),
            _ => None,
        };
        op
    }

    fn operate(block: &mut Block, operation: Operation) {
        use Operation::*;
        match operation {
            Left => block.left(),
            Right => block.right(),
            Down => block.down(),
            LRot => block.rotate_left(),
            RRot => block.rotate_right(),
        };
    }

    pub fn step(&mut self, input: Option<char>) {
        if let Some(ref mut block) = self.block {
            let operation = Self::parse(input);
            if let Some(op) = operation {
                let prev_rot = block.rotate.clone();
                let prev_coord = block.coord.clone();
                Game::operate(block, op);
                if let Some(field_with_block) = self.field.arrange_with_block(&block) {
                    let formatted = Field::format_field(&field_with_block);
                    Game::render(formatted);
                } else {
                    block.rotate = prev_rot;
                    block.coord = prev_coord;
                }
            }
        }
    }
    // FIXME: block.down()に対してバインドして、updateも呼びたい、レンダーは外部で実行したい
    pub fn drop(&mut self) {
        let mut formatted = Field::format_field(&self.field.field);
        if let Some(ref mut block) = self.block {
            let prev_rot = block.rotate.clone();
            let prev_coord = block.coord.clone();
            block.down();
            if let Some(field_with_block) = self.field.arrange_with_block(&block) {
                formatted = Field::format_field(&field_with_block);
            } else {
                block.rotate = prev_rot;
                block.coord = prev_coord; 
                self.update_state();
            }
        }
        Game::render(formatted);
    }

    fn update_state(&mut self) {
        let block = self.block.as_ref().unwrap();
        if let Some(field_with_block) = self.field.arrange_with_block(block) {
            self.field.set(field_with_block);
            self.field.update();
        }
        self.new_block();
    }

    pub fn update(&mut self, input: Option<char>) {
        let now = std::time::Instant::now();
        
        // ユーザー入力があれば処理
        if let Some(_) = input {
            self.step(input);
        }
        
        // 一定時間経過したらドロップ
        if now.duration_since(self.last_drop_time) >= self.drop_interval {
            self.drop();
            self.last_drop_time = now;
        }
    }

    fn render(formatted: String) {
        execute!(
            io::stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0)
        ).unwrap();
        print!("{}", formatted);
        io::stdout().flush().unwrap();
    }
}


pub fn show_with_block(field: &Field, block: &Block) {
    let rendered = field.arrange_with_block(&block);
    let formatted = Field::format_field(rendered.as_ref().unwrap());
    println!("{}", formatted);
}

pub fn show(field: &Field) {
    let formatted = Field::format_field(&field.field);
    println!("{}", formatted);
}



const CLOCK_TIME: Duration = Duration::from_millis(500);
use std::{thread, time};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor::MoveTo,
};
fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut game = Game::new(10, 22, CLOCK_TIME);
    game.new_block();

    loop {
        // 非ブロッキングでユーザー入力をチェック
        let input = if event::poll(std::time::Duration::from_millis(0))? {
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
        
        thread::sleep(time::Duration::from_millis(16)); // 約60FPS
    }
    disable_raw_mode()?;
    Ok(())
}

#[test]
fn load_with_str() {
    let formatted = 
"
[]. . . . . []
[]. . . . . []
[]. . . []. []
[]. . . . . []
[][][][][][][]
";
    let field = Field::init_with_str(formatted);
    print!("{}", Field::format_field(&field));
}

#[test]
fn render() {
    let mut field = Field::new(5, 4);
    let mut grid = field.field.clone();
    grid[2][4] = true;
    field.set(grid);
    let formatted = Field::format_field(&field.field);
    assert_eq!(format!("\n{}", formatted),
"
[]. . . . . []
[]. . . . . []
[]. . . []. []
[]. . . . . []
[][][][][][][]
"
    );
}

#[test]
fn render_with_block() {
    let mut field = Field::new(5, 4);
    let block = Block::new(BlockShape::J, (22, 10));
    field.field[2][4] = true;
    let rendered = field.arrange_with_block(&block);
    field.set(rendered.unwrap());
    let formatted = Field::format_field(&field.field);
    print!("{}", formatted);
    assert_eq!(format!("\n{}", formatted),
"
[][]. . . . []
[][][][]. . []
[]. . . []. []
[]. . . . . []
[][][][][][][]
"
    );
}

#[test]
fn right() {
    let mut field = Field::new(5, 4);
    let mut block = Block::new(BlockShape::J, (22, 10));
    field.field[2][4] = true;
    block.right();
    let rendered = field.arrange_with_block(&block);
    field.set(rendered.unwrap());
    let formatted = Field::format_field(&field.field);
    println!("{}", formatted);
    assert_eq!(format!("\n{}", formatted),
"
[]. []. . . []
[]. [][][]. []
[]. . . []. []
[]. . . . . []
[][][][][][][]
"
    );
}