use std::{io::{self, Write}, time::Duration};
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
    coord: (usize, usize),
    rotate: usize,
    shape: Vec<Grid>,
}

impl Block {
    pub fn rotate_left(&mut self) { 
        self.rotate -= 1;
        self.rotate %= 4;
    }

    pub fn rotate_right(&mut self) {
        self.rotate += 1;
        self.rotate %= 4;
    }

    pub fn down(&mut self) {
        self.coord.1 += 1
    }

    pub fn left(&mut self) {
        self.coord.0 -= 1;
    }

    pub fn right(&mut self) {
        self.coord.0 += 1;
    }

    pub fn new(shape_type: BlockShape) -> Self {
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
            coord: (1, 0),
            rotate: 0,
            shape,
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
        for x in 0..width + 2 {
            field[height][x] = true;
        }
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

    fn arrange_with_block(&self, block: &Block) -> Option<Grid> {
        let mut output = self.field.clone();
        let cols_block = block.shape[0].len();
        let rows_block = block.shape[0][0].len();
        for y in 0..cols_block {
            for x in 0..rows_block {
                let ref mut elem = output[block.coord.1 + y][block.coord.0 + x];
                let block_elem = block.shape[block.rotate][y][x];
                if *elem && block_elem {
                    return None;
                }
                *elem |= block_elem;
            }
        }
        Some(output)
    }

    fn format_field(field: &Grid) -> String {
        let field_fmt = field.iter().map(|w_vec| {
            let inside = w_vec.iter().map(|elem| {
                if *elem {
                   "[]"
                } else {
                    ". "
                }
            }).collect::<String>();
            format!("{}\n", &inside)
        }).collect::<String>();
        field_fmt
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
}

impl Game {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            field: Field::new(width, height),
            block: None,
        }
    }

    pub fn new_block(&mut self) {
        let shape: BlockShape = rand::random();
        self.block = Some(Block::new(shape));
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
        //if let Some(ref mut block) = self.block {
            match operation {
                Left => block.left(),
                Right => block.right(),
                Down => block.down(),
                LRot => block.rotate_left(),
                RRot => block.rotate_right(),
            };
        //}
    }

    pub fn step(&mut self, input: Option<char>) {
        if let Some(ref mut block) = self.block {
            let operation = Self::parse(input);
            if let Some(op) = operation {
                let prev_coord = block.coord.clone();
                Game::operate(block, op);
                if let Some(field_with_block) = self.field.arrange_with_block(&block) {
                    let formatted = Field::format_field(&field_with_block);
                    Game::render(formatted);
                } else {
                    block.coord = prev_coord;
                }
            }
        }
    }

    fn render(formatted: String) {
        // 画面をクリア
        print!("{esc}c", esc = 27 as char);
        // フォーマットされたフィールドを出力
        print!("{}", formatted);
        // 標準出力をフラッシュ
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

use std::{thread, time};

const CLOCK_TIME: Duration = time::Duration::from_millis(500);
fn main() {
    let mut game = Game::new(10, 22);
    game.new_block();
    for _ in 0..5 {
        game.step(Some('d'));
        thread::sleep(CLOCK_TIME);
        game.step(Some('s'));
        thread::sleep(CLOCK_TIME);
    }
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
    let mut grid = field.field.clone();
    let block = Block::new(BlockShape::J);
    grid[2][4] = true;
    let rendered = Field::arrange_with_block(&grid, &block);
    field.set(rendered.unwrap());
    let formatted = Field::format_field(&field.field);
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
    let mut block = Block::new(BlockShape::J);
    field.field[2][4] = true;
    block.right();
    let rendered = Field::arrange_with_block(&field.field, &block);
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