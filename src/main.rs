struct Game {
    board: Vec<Vec<Tile>>,
}

enum Piece {
    Pawn(Owner),
    Rook(Owner),
    Knight(Owner),
    Bishop(Owner),
    Queen(Owner),
    King(Owner),
}

enum Owner {
    Black,
    White,
}

struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    fn new(x: i32, y: i32) -> Option<Pos> {
        if (x >= BOARD_SIZE) | (x < 0) | (y >= BOARD_SIZE) | (y < 0) {
            None
        } else {
            Some(Pos { x: x, y: y })
        }
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
    fn calculate_raw_moves(&self, pos: Pos) -> Vec<Pos> {
        let mut moves: Vec<Pos> = Vec::new();
        let vert_horiz = || {
            for i in 0..BOARD_SIZE {
                moves.push(Pos::new(pos.x, i));
                moves.push(Pos::new(i, pos.y));
            }
        };
        let diag = || {
            for i in 0..BOARD_SIZE {
                moves.push(Pos::new(i, i));
                moves.push(Pos::new(BOARD_SIZE - i, i));
            }
        };
        match self {
            // TODO: add logic for start-of-game condition (able to move 2 in positive y)
            Piece::Pawn(owner) => {
                moves.push(Pos::new(pos.x, pos.y + 1));
            }
            Piece::Rook(owner) => {
                let mut pos_test: Option<Pos>;
                vert_horiz;
            }
            Piece::Knight(owner) => {
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
            Piece::Bishop(owner) => {
                diag;
            }
            Piece::Queen(owner) => {
                vert_horiz;
                diag;
            }
            Piece::King(owner) => moves,
        }
        moves.retain(|x| x != &pos);
        moves
    }
}

struct Tile {
    piece: Option<Piece>,
    shade: Option<Owner>,
    pos: Pos,
}

impl Tile {
    fn shade(&mut self, owner: Owner) {
        self.shade = Some(owner);
    }
}

const BOARD_SIZE: i32 = 8;

fn main() {}
