// Some notes:
// I'm using usizes to index all piece positions. I feel like converting to integral types could be a headache,
// but since I'm indexing so much I think it's the lesser of two evils.

use core::fmt;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const BOARD_SIZE: usize = 8;

#[derive(Debug)]
enum MoveError {
    InvalidSource,
    InvalidDest,
    Check,
}

enum Dir {
    neg,
    null,
    pos,
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveError::InvalidSource => write!(f, "source tile invalid"),
            MoveError::InvalidDest => write!(f, "destination tile invalid"),
            MoveError::Check => write!(f, "puts owner's king in check"),
        }
    }
}

impl std::error::Error for MoveError {}

struct Game {
    board: Vec<Vec<Tile>>,
}

impl Game {
    fn build_game() -> Game {
        let mut new = Game {
            board: vec![vec![Tile::blank(); BOARD_SIZE]; BOARD_SIZE],
        };
        new.init();
        new
    }

    fn general_move(pos: (usize, usize), dir: (Dir, Dir), dist: (usize, usize)) -> (usize, usize) {
        let mut new_pos = pos;
        new_pos.0 = match dir.0 {
            Dir::neg => pos.0 - dist.0,
            Dir::null => pos.0,
            Dir::pos => pos.0 + dist.0,
        };
        new_pos.1 = match dir.1 {
            Dir::neg => pos.1 - dist.1,
            Dir::null => pos.1,
            Dir::pos => pos.1 + dist.1,
        };
        new_pos
    }

    fn calc_moves(&mut self, pos: (usize, usize)) {
        let mut tile = self.get_mut_tile(pos).expect("pos should be a valid index");
        let piece = &tile.piece.expect("Tile shouldn't be empty");
        // Defined moves
        let r = |p| -> (usize, usize) { Game::general_move(p, (Dir::pos, Dir::null), (1, 0)) };
        let l = |p| -> (usize, usize) { Game::general_move(p, (Dir::neg, Dir::null), (1, 0)) };
        let d = |p| -> (usize, usize) { Game::general_move(p, (Dir::null, Dir::pos), (0, 1)) };
        let u = |p| -> (usize, usize) { Game::general_move(p, (Dir::null, Dir::neg), (0, 1)) };
        // Diagonal functions - up-right, up-left, down-right, down-left
        let dur = |p| -> (usize, usize) { Game::general_move(p, (Dir::pos, Dir::neg), (1, 1)) };
        let dul = |p| -> (usize, usize) { Game::general_move(p, (Dir::neg, Dir::neg), (1, 1)) };
        let ddr = |p| -> (usize, usize) { Game::general_move(p, (Dir::pos, Dir::pos), (1, 1)) };
        let ddl = |p| -> (usize, usize) { Game::general_move(p, (Dir::neg, Dir::pos), (1, 1)) };
        match piece {
            Piece::Pawn => {}
            Piece::Rook => {
                let move_fns: Vec<&dyn Fn((usize, usize)) -> (usize, usize)> = vec![&r, &l, &d, &u];
                for func in move_fns {
                    let test_tile = self.get_mut_tile(func(tile.pos));
                }
                // conditions: new tile needs to be in-bounds (stop when it's not)
                // conditions2: stop when you encounter another piece (if member of opposing side, include;
                // if not, dont)
            }
        }
    }

    // the choice of indexing into a tile vs a piece seems arbitrary, so I'm going to choose the thing I
    // know exists given a valid index, rather than a piece which might not exist at that valid index.
    fn get_tile(&self, pos: (usize, usize)) -> Option<&Tile> {
        self.board.get(pos.0).and_then(|x| x.get(pos.1))
    }

    fn get_mut_tile(&mut self, pos: (usize, usize)) -> Option<&mut Tile> {
        self.board.get_mut(pos.0).and_then(|x| x.get_mut(pos.1))
    }

    // Moves desired piece. Makes no checks.
    fn move_piece(&mut self, src: (usize, usize), dest: (usize, usize)) {
        let src_piece = self.board[src.0][src.1].piece.take();
        self.board[dest.0][dest.1].piece = src_piece;
    }

    // Assumes valid src and dest indices
    fn validate_and_move(
        &mut self,
        src: (usize, usize),
        dest: (usize, usize),
    ) -> Result<(), MoveError> {
        let src_tile = self.get_tile(src).expect("src should be a valid index");
        match &src_tile.piece {
            Some(src_piece) => {
                // if src_piece.calculate_raw_moves(src).contains(&dest) {
                //     self.move_piece(src, dest);
                //     Ok(())
                // } else {
                //     Err(MoveError::InvalidDest)
                // }
            }
            None => Err(MoveError::InvalidSource),
        }
    }
    // Load all pieces from csv file
    fn init(&mut self) {
        let file = File::open("initial_positions.csv").expect("file not found");
        let reader = BufReader::new(file);
        for line in reader.lines().skip(1) {
            let data: Vec<String> = line
                .unwrap()
                .split(',')
                .map(|x| x.parse::<String>().unwrap())
                .collect();
            let pos_x = data[1].parse::<usize>().unwrap();
            let pos_y = data[2].parse::<usize>().unwrap();
            let tile: &mut Tile = &mut self.board[pos_x][pos_y];
            tile.pos = (pos_x, pos_y);
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
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Clone, PartialEq)]
enum Owner {
    Black,
    White,
}

impl Piece {
    // Calculates moves only with regard to individual piece's movement rules
    // TODO: I wonder if I can simplify this with iterator methods?
    // fn calculate_raw_moves(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
    //     let mut moves: Vec<(usize, usize)> = Vec::new();
    //     // Defines reused behaviors for several pieces; code 0 is for horizontal
    //     // and vertical movement, code 1 for diagonals
    //     let mut large_move = |x: usize| {
    //         if x == 0 {
    //             for i in 0..BOARD_SIZE {
    //                 moves.push((pos.0, i));
    //                 moves.push((i, pos.1));
    //             }
    //         } else {
    //             for i in 0..BOARD_SIZE {
    //                 moves.push((i, i));
    //                 moves.push((BOARD_SIZE - i, i));
    //             }
    //         }
    //     };
    //     match self {
    //         // TODO: add logic for start-of-game condition (able to move 2 in
    //         // positive y)
    //         Piece::Pawn => {
    //             moves.push((pos.0, pos.1 + 1));
    //         }
    //         Piece::Rook => {
    //             large_move(0);
    //         }
    //         Piece::Knight => {
    //             let list: Vec<(i32, i32)> = vec![
    //                 (1, 2),
    //                 (2, 1),
    //                 (-1, -2),
    //                 (-2, -1),
    //                 (-1, 2),
    //                 (-2, 1),
    //                 (1, -2),
    //                 (2, -1),
    //             ];
    //             for i in list {
    //                 moves.push((pos.0 + i.0 as usize, pos.1 + i.1 as usize));
    //             }
    //         }
    //         Piece::Bishop => {
    //             large_move(1);
    //         }
    //         Piece::Queen => {
    //             large_move(0);
    //             large_move(1);
    //         }
    //         Piece::King => {
    //             let list: Vec<(i32, i32)> = vec![
    //                 (-1, -1),
    //                 (-1, 0),
    //                 (-1, 1),
    //                 (0, -1),
    //                 (0, 0),
    //                 (0, 1),
    //                 (1, -1),
    //                 (1, 0),
    //                 (1, 1),
    //             ];
    //             for i in list {
    //                 moves.push((pos.0 + i.0 as usize, pos.1 + i.1 as usize));
    //             }
    //         }
    //     }
    //     // cleanses list of current position as well as out-of-bounds positions
    //     moves.retain(|x| {
    //         (x != &pos) & (x.0 > 0) & (x.1 > 0) & (x.0 < BOARD_SIZE) & (x.1 < BOARD_SIZE)
    //     });
    //     moves
    // }
}
// a movable container for a piece (which could not exist) and a "shade"... wait
// Tile shouldn't really ever be cloned, except during board creation
#[derive(Clone)]
struct Tile {
    moves: Vec<(usize, usize)>,
    pos: (usize, usize),
    piece: Option<Piece>,
    piece_str: String,
    owner: Option<Owner>,
    shade: Option<Owner>,
}

impl Tile {
    fn blank() -> Tile {
        Tile {
            moves: Vec::new(),
            pos: (0, 0),
            piece: None,
            piece_str: String::new(),
            owner: None,
            shade: None,
        }
    }
}

fn main() {
    let game = Game::build_game();
    game.print_board();
}
