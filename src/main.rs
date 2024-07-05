use std::vec;

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
            coord: (0, 0),
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
        Self {
            field: vec![vec![false; width]; height],
        }
    }

    pub fn set(&mut self, field: Grid){
        self.field = field;
    }

    fn arrange_with_block(field: &Grid, block: &Block) -> Option<Grid> {
        let mut output = field.clone();
        let cols_block = block.shape[0].len();
        let rows_block = block.shape[0][0].len();
        for y in 0..cols_block {
            for x in 0..rows_block {
                let ref mut elem = output[block.coord.1 + y][block.coord.0 + x];
                let block_elem = block.shape[block.rotate][y][x];
                if *elem && block_elem {
                    return None;
                }
                *elem = block_elem;
            }
        }
        Some(output)
    }

    fn format_field(field: &Grid) -> String {
        let rows = field[0].len();
        let field_fmt = field.iter().map(|w_vec| {
            let inside = w_vec.iter().map(|elem| {
                if *elem {
                   "[]"
                } else {
                    ". "
                }
            }).collect::<String>();
            format!("[]{}[]\n", &inside)
        }).collect::<String>();
        format!("{}{}\n", field_fmt, "[]".repeat(rows + 2))
    }
}

struct Game {
    field: Field,
    block: Block,
}

impl Game {

}


pub fn show_with_block(field: &Field, block: &Block) {
    let rendered = Field::arrange_with_block(&field.field, &block);
    let formatted = Field::format_field(rendered.as_ref().unwrap());
    println!("{}", formatted);
}

pub fn show(field: &Field) {
    let formatted = Field::format_field(&field.field);
    println!("{}", formatted);
}

fn main() {
    let field = Field::new(10, 22);
    println!("{}", Field::format_field(&field.field));
    let block = Block::new(BlockShape::I);
    show_with_block(&field, &block);
}

#[test]
fn render() {
    let mut field = Field::new(5, 4);
    let mut grid = field.field.clone();
    grid[2][4] = true;
    field.set(grid);
    show(&field);
    // ---- collision stdout ----
    // []. . . . . []
    // []. . . . . []
    // []. . . . [][]
    // []. . . . . []
    // [][][][][][][]
    assert_eq!(field.field, vec![
        vec![false, false, false, false, false],
        vec![false, false, false, false, false],
        vec![false, false, false, false, true],
        vec![false, false, false, false, false],
        ]
    )
}

#[test]
fn render_with_block() {
    let mut field = Field::new(5, 4);
    let mut grid = field.field.clone();
    let block = Block::new(BlockShape::J);
    grid[2][4] = true;
    let rendered = Field::arrange_with_block(&grid, &block);
    field.set(rendered.unwrap());
    show(&field);
    // ---- render_with_block stdout ----
    // [][]. . . . []
    // [][][][]. . []
    // []. . . . [][]
    // []. . . . . []
    // [][][][][][][]
    assert_eq!(field.field, vec![
        vec![true, false, false, false, false],
        vec![true, true, true, false, false],
        vec![false, false, false, false, true],
        vec![false, false, false, false, false],
        ]
    )
}

#[test]
fn right() {
    let mut field = Field::new(5, 4);
    let mut grid = field.field.clone();
    let block = Block::new(BlockShape::J);
    grid[2][4] = true;
    let rendered = Field::arrange_with_block(&grid, &block);
    field.set(rendered.unwrap());
    show(&field);
    // ---- render_with_block stdout ----
    // [][]. . . . []
    // [][][][]. . []
    // []. . . . [][]
    // []. . . . . []
    // [][][][][][][]
    assert_eq!(field.field, vec![
        vec![true, false, false, false, false],
        vec![true, true, true, false, false],
        vec![false, false, false, false, true],
        vec![false, false, false, false, false],
        ]
    )
}