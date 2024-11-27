// Some notes:
// I'm using usizes to index all piece positions. I feel like converting to integral types could be a headache,
// but since I'm indexing so much I think it's the lesser of two evils.

use core::fmt;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    os::fd::OwnedFd,
};

const BOARD_SIZE: usize = 8;

#[derive(Debug)]
enum MoveError {
    InvalidSource,
    InvalidDest,
    Check,
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
    white_king: Option<Tile>,
    black_king: Option<Tile>,
    is_start: Option<Owner>,
}

impl Game {
    fn build_game() -> Game {
        let mut new = Game {
            board: vec![vec![Tile::blank(); BOARD_SIZE]; BOARD_SIZE],
            white_king: None,
            black_king: None,
            is_start: Some(Owner::White),
        };
        new.init();
        new
    }

    //
    fn general_move(src: (usize, usize), shift: (i32, i32)) -> (usize, usize) {
        let mut new_pos = (src.0 as i32, src.1 as i32);
        new_pos.0 += shift.0;
        new_pos.1 += shift.1;
        (new_pos.0 as usize, new_pos.1 as usize)
    }

    fn calc_moves(&mut self, pos: (usize, usize)) {
        let src_tile = self.get_tile(pos).expect("pos should be a valid index");
        let piece = src_tile.piece.as_ref().expect("tile shouldn't be empty");
        let mut new_moves: Vec<(usize, usize)> = Vec::new();
        match piece {
            Piece::Pawn => {
                let owner = src_tile.owner.as_ref().expect("tile shouldn't be empty");
                let shift = match owner {
                    Owner::Black => 1,
                    Owner::White => -1,
                };
                let pawn_moves = vec![
                    Game::general_move(src_tile.pos, (0, shift)),
                    Game::general_move(src_tile.pos, (0, shift + 1)),
                ];
                let take_moves = vec![
                    Game::general_move(src_tile.pos, (-1, shift)),
                    Game::general_move(src_tile.pos, (1, shift)),
                ];
                for m in pawn_moves {
                    if self.get_tile(m).is_some_and(|t| t.owner != src_tile.owner) {
                        new_moves.push(m);
                    }
                }
                for m in take_moves {
                    let move_tile = self.get_tile(m);
                    if move_tile.is_some_and(|t| t.owner.clone().is_some_and(|o| &o != owner)) {
                        new_moves.push(m);
                    }
                }
            }
            Piece::Knight => {
                let knight_moves = vec![
                    Game::general_move(src_tile.pos, (1, 2)),
                    Game::general_move(src_tile.pos, (2, 1)),
                    Game::general_move(src_tile.pos, (-1, 2)),
                    Game::general_move(src_tile.pos, (-2, 1)),
                    Game::general_move(src_tile.pos, (1, -2)),
                    Game::general_move(src_tile.pos, (2, -1)),
                    Game::general_move(src_tile.pos, (-1, -2)),
                    Game::general_move(src_tile.pos, (-2, -1)),
                ];
                for m in knight_moves {
                    if self.get_tile(m).is_some_and(|t| t.owner != src_tile.owner) {
                        new_moves.push(m);
                    }
                }
            }
            Piece::King => {
                let king_moves = vec![
                    Game::general_move(src_tile.pos, (-1, -1)),
                    Game::general_move(src_tile.pos, (0, -1)),
                    Game::general_move(src_tile.pos, (1, -1)),
                    Game::general_move(src_tile.pos, (-1, 0)),
                    Game::general_move(src_tile.pos, (1, 0)),
                    Game::general_move(src_tile.pos, (-1, 1)),
                    Game::general_move(src_tile.pos, (0, 1)),
                    Game::general_move(src_tile.pos, (1, 1)),
                ];

                for m in king_moves {
                    if self.get_tile(m).is_some_and(|t| t.owner != src_tile.owner) {
                        new_moves.push(m);
                    }
                }
            }
            other => {
                let r = |p| -> (usize, usize) { Game::general_move(p, (1, 0)) };
                let l = |p| -> (usize, usize) { Game::general_move(p, (-1, 0)) };
                let d = |p| -> (usize, usize) { Game::general_move(p, (0, 1)) };
                let u = |p| -> (usize, usize) { Game::general_move(p, (0, -1)) };
                // Diagonal functions - up-right, up-left, down-right, down-left
                let dur = |p| -> (usize, usize) { Game::general_move(p, (1, -1)) };
                let dul = |p| -> (usize, usize) { Game::general_move(p, (-1, -1)) };
                let ddr = |p| -> (usize, usize) { Game::general_move(p, (1, 1)) };
                let ddl = |p| -> (usize, usize) { Game::general_move(p, (-1, 1)) };

                let move_fns: Vec<&dyn Fn((usize, usize)) -> (usize, usize)> = match other {
                    Piece::Rook => vec![&r, &l, &d, &u],
                    Piece::Bishop => vec![&dur, &dul, &ddr, &ddl],
                    _ => vec![&r, &l, &d, &u, &dur, &dul, &ddr, &ddl],
                };
                for func in move_fns {
                    let mut pos = func(src_tile.pos);
                    while let Some(tile) = self.get_tile(pos) {
                        if tile.owner.is_none() {
                            new_moves.push(pos);
                        } else if tile.owner.is_some() && tile.owner == src_tile.owner {
                            break;
                        } else {
                            new_moves.push(pos);
                            break;
                        }
                        pos = func(pos);
                    }
                }
            }
        }
        self.get_mut_tile(pos).unwrap().moves = new_moves;
    }

    // the choice of indexing into a tile vs a piece seems arbitrary, so I'm going to choose the thing I
    // know exists given a valid index, rather than a piece which might not exist at that valid index.
    fn get_tile(&self, pos: (usize, usize)) -> Option<&Tile> {
        self.board.get(pos.0).and_then(|x| x.get(pos.1))
    }

    fn get_mut_tile(&mut self, pos: (usize, usize)) -> Option<&mut Tile> {
        self.board.get_mut(pos.0).and_then(|x| x.get_mut(pos.1))
    }

    // Moves desired piece. Assumes src tile carries a piece.
    fn move_piece(&mut self, src: (usize, usize), dest: (usize, usize)) {
        let src_piece = self
            .get_mut_tile(src)
            .expect("src should be a valid index")
            .piece
            .take();
        self.get_mut_tile(dest)
            .expect("dest should be a valid index")
            .piece = src_piece;
    }

    // Assumes valid src and dest indices
    fn validate_and_move(
        &mut self,
        src: (usize, usize),
        dest: (usize, usize),
    ) -> Result<(), MoveError> {
        self.calc_moves(src);
        let src_tile = self.get_mut_tile(src).expect("src should be a valid index");
        if src_tile.piece.is_some() {
            if src_tile.moves.contains(&dest) {
                self.move_piece(src, dest);
                self.is_start = match self.is_start {
                    Some(Owner::White) => Some(Owner::Black),
                    Some(Owner::Black) => None,
                    None => None,
                };
                Ok(())
            } else {
                Err(MoveError::InvalidDest)
            }
        } else {
            Err(MoveError::InvalidSource)
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
