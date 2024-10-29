use std::{
    clone,
    fs::File,
    io::{BufRead, BufReader},
};

const BOARD_SIZE: i32 = 8;

struct Game {
    board: Vec<Vec<Tile>>,
}

struct Line {
    code: i32,
    x: i32,
    y: i32,
}

impl Game {
    fn build_game() -> Game {
        let mut new = Game {
            board: vec![vec![Tile::blank(); BOARD_SIZE as usize]; BOARD_SIZE as usize],
        };
        new.init();
        new
    }
    // Load all pieces from csv file
    fn init(&mut self) {
        let file = File::open("initial_positions.csv").expect("file not found");
        let reader = BufReader::new(file);
        for line in reader.lines().skip(1) {
            let mut data: Vec<String> = line
                .unwrap()
                .split(',')
                .map(|x| x.parse::<String>().unwrap())
                .collect();
            // remove header line

            let tile: &mut Tile = &mut self.board[data[1].parse::<usize>().unwrap()]
                [data[2].parse::<usize>().unwrap()];
            tile.piece_str = data.get(0).unwrap().to_owned();
            tile.piece = (|| match tile.piece_str.as_str() {
                "pawn" => Some(Piece::Pawn),
                "rook" => Some(Piece::Rook),
                "knight" => Some(Piece::Knight),
                "bishop" => Some(Piece::Bishop),
                "queen" => Some(Piece::Queen),
                "king" => Some(Piece::King),
                _ => None,
            })();
            tile.owner = (|| match data[3].as_str() {
                "black" => Some(Owner::Black),
                "white" => Some(Owner::White),
                _ => None,
            })();
        }
    }

    fn print_board(&self) {
        for i in &self.board {
            for j in i {
                print!(
                    "{:<6} {:<7}",
                    (|| match j.owner {
                        Some(Owner::Black) => "Black",
                        Some(Owner::White) => "White",
                        None => "N",
                    })(),
                    j.piece_str
                )
            }
            print!("\n\n");
        }
    }
}

#[derive(Clone)]
enum Piece {
    Pawn, // parse code 1
    Rook,
    Knight,
    Bishop,
    Queen,
    King, // parse code 7
}

#[derive(Clone)]
enum Owner {
    Black,
    White,
}

#[derive(Clone)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn new(x: i32, y: i32) -> Pos {
        let new = Pos { x: x, y: y };
        new
    }
}

impl PartialEq for Pos {
    fn ne(&self, other: &Self) -> bool {
        (self.x != other.x) | (self.y != other.y)
    }
    fn eq(&self, other: &Self) -> bool {
        (self.x == other.x) & (self.y == other.y)
    }
}
impl Eq for Pos {}

impl Piece {
    // Calculates moves only with regard to individual piece's movement rules
    // TODO: I wonder if I can simplify this with iterator methods?
    fn calculate_raw_moves(&self, pos: Pos) -> Vec<Pos> {
        let mut moves: Vec<Pos> = Vec::new();
        // Defines reused behaviors for several pieces; code 0 is for horizontal
        // and vertical movement, code 1 for diagonals
        let mut large_move = |x| {
            if x == 0 {
                for i in 0..BOARD_SIZE {
                    moves.push(Pos::new(pos.x, i));
                    moves.push(Pos::new(i, pos.y));
                }
            } else {
                for i in 0..BOARD_SIZE {
                    moves.push(Pos::new(i, i));
                    moves.push(Pos::new(BOARD_SIZE - i, i));
                }
            }
        };
        match self {
            // TODO: add logic for start-of-game condition (able to move 2 in
            // positive y)
            Piece::Pawn => {
                moves.push(Pos::new(pos.x, pos.y + 1));
            }
            Piece::Rook => {
                large_move(0);
            }
            Piece::Knight => {
                let list: Vec<(i32, i32)> = vec![
                    (1, 2),
                    (2, 1),
                    (-1, -2),
                    (-2, -1),
                    (-1, 2),
                    (-2, 1),
                    (1, -2),
                    (2, -1),
                ];
                for i in list {
                    moves.push(Pos::new(pos.x + i.0, pos.y + i.1));
                }
            }
            Piece::Bishop => {
                large_move(1);
            }
            Piece::Queen => {
                large_move(0);
                large_move(1);
            }
            Piece::King => {
                for i in -1..2 {
                    for j in -1..2 {
                        moves.push(Pos::new(pos.x + i, pos.y + j));
                    }
                }
            }
        }
        moves.retain(|x| {
            (x != &pos) & (x.x > 0) & (x.y > 0) & (x.x < BOARD_SIZE) & (x.y < BOARD_SIZE)
        });
        moves
    }
}
#[derive(Clone)]
struct Tile {
    piece: Option<Piece>,
    piece_str: String,
    owner: Option<Owner>,
    shade: Option<Owner>,
    pos: Pos,
}

// impl Clone for Tile {
//     fn clone(&self) -> Self {
//         Self {
//             piece: self.piece,
//             piece_str: self.piece_str,
//             owner: self.owner,
//             shade: self.shade,
//             pos: self.pos,
//         }
//     }
// }

impl Tile {
    fn blank() -> Tile {
        Tile {
            piece: None,
            piece_str: String::new(),
            owner: None,
            // TODO: figure out if pos is a necessary abstraction
            shade: None,
            pos: Pos::new(0, 0),
        }
    }
}

fn main() {
    let game = Game::build_game();
    game.print_board();
}
